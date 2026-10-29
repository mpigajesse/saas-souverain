"""Logique métier de l'espace tenant, appelée par l'API REST.

Regroupe ce qui était partagé entre les anciennes vues web Django (retirées en
V3, la console est désormais en React) et l'API : création de compte,
contextes d'accueil et d'espace PME, génération du docker-compose et des
installeurs.
"""
from datetime import date, datetime, timedelta
import unicodedata

from django.conf import settings
from django.contrib.auth.models import User
from django.http import HttpResponse
from django.shortcuts import render
from django.utils.text import slugify

from licenses.models import License
from .models import Tenant
from .manifeste import pour_compose
from .pile import images_pile

# Types de machine d'un cluster PME (ADR-005).
TYPES_MACHINE = ('serveur', 'temoin')


def _create_tenant_account(cleaned_data: dict) -> tuple[User, Tenant]:
    """Crée le `User` Django, le `Tenant` lié et sa licence d'essai.

    Appelé par `POST /api/auth/register/`, après validation par
    `InscriptionForm` (email unique, mots de passe identiques).
    """
    d = cleaned_data

    # Créer le compte Django
    username = d['email'].split('@')[0][:150]
    base = username
    i = 1
    while User.objects.filter(username=username).exists():
        username = f'{base}{i}'
        i += 1
    user = User.objects.create_user(
        username=username,
        email=d['email'],
        password=d['password'],
        first_name='',
    )

    # Créer le tenant lié
    tenant = Tenant.objects.create(
        name=d['name'],
        email=d['email'],
        phone=d.get('phone', ''),
        address=d.get('address', ''),
        employee_count=d['employee_count'],
        is_active=True,
        user=user,
    )

    # Licence trial automatique (30 jours, starter, sièges = nb employés)
    License.objects.create(
        tenant=tenant,
        plan='starter',
        seats=d['employee_count'],
        starts_at=date.today(),
        expires_at=date.today() + timedelta(days=30),
        is_active=True,
    )

    return user, tenant


def build_bienvenue_context(tenant: Tenant) -> dict:
    """Contexte de l'accueil post-inscription (`GET /api/tenants/me/onboarding/`)."""
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    active_devices = tenant.devices.filter(is_active=True).count()
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    return {
        'tenant': tenant,
        'active_license': active_license,
        'active_devices': active_devices,
        'relay_url': relay_url,
    }


def _pgadmin_url(tenant) -> str:
    """URL LAN de pgAdmin (port 5050, exposé par docker-compose.yml.tmpl).

    Dérivée de l'IP du nœud primaire connu (ou, à défaut, du dernier nœud actif
    annoncé) — sans nœud annoncé, aucune IP n'est connue et l'URL est vide.
    Les identifiants pgAdmin ne sont PAS connus du portail : ils sont générés
    sur le réseau de la PME (ADR-005, décision 4).
    """
    device = (
        tenant.devices.filter(is_active=True, node_kind='serveur', node_role='primary').order_by('-last_seen').first()
        or tenant.devices.filter(is_active=True, node_kind='serveur').order_by('-last_seen').first()
    )
    if device and device.node_addr:
        host = device.node_addr.split(':')[0]
        return f"http://{host}:5050"
    return ""


def besoin_temoin(tenant) -> bool:
    """Vrai si le cluster compte exactement deux serveurs actifs et aucun
    témoin : sans témoin, etcd n'a pas de majorité tolérant la perte d'une
    machine, et la panne de l'un des serveurs bloque toute écriture
    (ADR-004, décision 1)."""
    actifs = tenant.devices.filter(is_active=True)
    return (
        actifs.filter(node_kind='serveur').count() == 2
        and not actifs.filter(node_kind='temoin').exists()
    )


def build_mon_espace_context(tenant) -> dict:
    """Contexte de l'espace PME (`GET /api/tenants/me/`)."""
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    devices = tenant.devices.filter(is_active=True).order_by('-last_seen')

    return {
        'tenant': tenant,
        'active_license': active_license,
        'devices': devices,
        'besoin_temoin': besoin_temoin(tenant),
    }


def _saas_host_port(request) -> tuple[str, str]:
    """Dérive host/port SaaS à partir de la requête.

    L'installeur sera téléchargé depuis la VM PME, donc `request.get_host()`
    retourne l'IP LAN du PC physique.
    """
    host_full = request.get_host()
    if ':' in host_full:
        saas_host, saas_port = host_full.rsplit(':', 1)
    else:
        saas_host = host_full
        saas_port = '443' if request.is_secure() else '80'
    return saas_host, saas_port


def _registry_host(request) -> str:
    """Hôte du registre Docker de l'éditeur, vu depuis la machine de la PME.

    Jamais codé en dur : l'IP de l'éditeur dépend de l'interface réseau et du
    réseau sur lequel tourne le banc de test. On reprend l'hôte par lequel la
    PME vient de joindre l'éditeur — s'il a servi à télécharger l'installeur,
    il servira aussi à tirer l'image.
    """
    saas_host, _ = _saas_host_port(request)
    return f'{saas_host}:{getattr(settings, "REGISTRY_PORT", "5000")}'


def application_hebergee(tenant: Tenant):
    """Application validée rattachée à la licence active du tenant, ou None
    (étape 5, ADR-009). Une application retirée n'est plus hébergée."""
    licence = (tenant.licenses.filter(is_active=True)
               .select_related('application').order_by('-created_at').first())
    if licence is None or licence.application is None:
        return None
    if licence.application.statut != licence.application.VALIDEE:
        return None
    return licence.application


def application_du_tenant(tenant: Tenant):
    """Manifeste de l'application externe hébergée pour ce tenant, ou None."""
    app = application_hebergee(tenant)
    return app.manifeste if app else None


def build_compose_response(request, tenant: Tenant, type_machine: str = 'serveur') -> HttpResponse:
    """Génère le docker-compose.yml d'une machine serveur ou témoin.

    Appelé par `GET /api/tenants/me/download/compose/` et par la voie PME
    `GET /api/tenants/pme/compose/`. Lève `pile.PileNonPubliee` si les images
    épinglées ne sont pas publiées : l'appelant répond 503.
    """
    gabarit = {
        'serveur': 'tenants/docker-compose.yml.tmpl',
        'temoin': 'tenants/docker-compose-temoin.yml.tmpl',
    }[type_machine]
    registre = _registry_host(request)
    manifeste = application_du_tenant(tenant) if type_machine == 'serveur' else None
    content = render(request, gabarit, {
        'tenant': tenant,
        'images': images_pile(registre),
        # Application externe hébergée (étape 3) : None si le tenant n'en a pas.
        'application': pour_compose(manifeste, registre) if manifeste else None,
        'relay_url': getattr(settings, 'RELAY_URL', 'http://localhost:8080'),
        'generated_at': datetime.now().strftime('%d/%m/%Y à %H:%M'),
    }).content.decode('utf-8')

    response = HttpResponse(content, content_type='application/x-yaml; charset=utf-8')
    response['Content-Disposition'] = 'attachment; filename="docker-compose.yml"'
    return response


def build_installeur_response(request, tenant: Tenant, os_type: str, type_machine: str = 'serveur') -> HttpResponse:
    """
    Génère l'installateur PME pour Windows (.bat) ou Linux (.sh), d'une machine
    serveur ou témoin. L'installateur installe Docker si besoin, déclare le
    registre de l'éditeur, télécharge la pile (images épinglées par digest) et
    la démarre ; les machines se coordonnent ensuite seules sur le réseau local.

    Appelé par `GET /api/tenants/me/download/installer/<os>/`. `os_type` doit
    valoir `'windows'` ou `'linux'`, `type_machine` `'serveur'` ou `'temoin'`
    — à valider par l'appelant.
    """
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    registry_host = _registry_host(request)
    saas_host, saas_port = _saas_host_port(request)

    # Nom replié en ASCII pur pour l'installeur Windows. `cmd` lit un fichier
    # .bat par décalage d'octets : un caractère multi-octets (accent, guillemet
    # français) lui fait perdre sa position après un `goto`, et le script part
    # en vrille ou s'arrête sans message. Le reste du gabarit est en ASCII pour
    # cette raison ; le nom du tenant est la seule donnée variable qui pourrait
    # en réintroduire.
    nom_ascii = (
        unicodedata.normalize('NFKD', tenant.name)
        .encode('ascii', 'ignore')
        .decode('ascii')
    )
    # Les métacaractères de `cmd` sont retirés en plus des accents : un nom
    # contenant « & » couperait la ligne `echo` en deux et cmd tenterait
    # d'exécuter la fin comme une commande ; « ( » et « ) » déséquilibreraient
    # les blocs ; « ! » serait mangé par EnableDelayedExpansion.
    nom_ascii = ''.join(' ' if c in '&<>|^%!()"' else c for c in nom_ascii)
    nom_ascii = ' '.join(nom_ascii.split()) or 'PME'

    ctx = {
        'tenant': tenant,
        'tenant_nom_ascii': nom_ascii,
        'type_machine': type_machine,
        'relay_url': relay_url,
        # Hôte du registre seul (sans le chemin d'image) : l'installeur doit le
        # déclarer en `insecure-registries` avant de pouvoir tirer l'image.
        'registry_host': registry_host,
        'saas_host': saas_host,
        'saas_port': saas_port,
        'generated_at': datetime.now().strftime('%d/%m/%Y à %H:%M'),
        # Variante sans accent pour l'en-tête du .bat : « à » est un caractère
        # multi-octets, et le .bat doit rester en ASCII pur (cf. `nom_ascii`).
        'generated_at_ascii': datetime.now().strftime('%d/%m/%Y %H:%M'),
    }

    suffixe = '-temoin' if type_machine == 'temoin' else ''
    if os_type == 'windows':
        content = render(request, 'tenants/install-windows.bat.tmpl', ctx).content.decode('utf-8')
        response = HttpResponse(content, content_type='application/octet-stream')
        response['Content-Disposition'] = f'attachment; filename="install-{slugify(tenant.name)}{suffixe}.bat"'
    else:
        content = render(request, 'tenants/install-linux.sh.tmpl', ctx).content.decode('utf-8')
        response = HttpResponse(content, content_type='application/octet-stream')
        response['Content-Disposition'] = f'attachment; filename="install-{slugify(tenant.name)}{suffixe}.sh"'

    return response
