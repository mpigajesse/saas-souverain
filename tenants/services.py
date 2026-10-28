"""Logique métier de l'espace tenant, appelée par l'API REST.

Regroupe ce qui était partagé entre les anciennes vues web Django (retirées en
V3, la console est désormais en React) et l'API : création de compte,
contextes d'accueil et d'espace PME, génération du docker-compose et des
installeurs.
"""
import hashlib
from datetime import date, datetime, timedelta
import unicodedata

from django.conf import settings
from django.contrib.auth.models import User
from django.http import HttpResponse
from django.shortcuts import render
from django.utils.text import slugify

from licenses.models import License
from .models import Tenant


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


def _derive_credentials(registration_token: str, tenant_name: str) -> dict:
    def _sha24(suffix: str) -> str:
        return hashlib.sha256(
            f"{registration_token}-{suffix}".encode()
        ).hexdigest()[:24]

    slug = slugify(tenant_name)
    return {
        'pgadmin_email': f"admin@pme{slug}.com",
        'pgadmin_password': _sha24('pgadmin'),
        'db_password': _sha24('pg'),
    }


def _pgadmin_url(tenant) -> str:
    """URL LAN de pgAdmin (port 5050, exposé par docker-compose.yml.tmpl).

    Dérivée de l'IP du nœud primaire connu (ou, à défaut, du dernier nœud actif
    annoncé) — sans nœud annoncé, aucune IP n'est connue et l'URL est vide.
    N'affecte pas `_derive_credentials`, qui reste la seule source des
    identifiants pgAdmin.
    """
    device = (
        tenant.devices.filter(is_active=True, node_role='primary').order_by('-last_seen').first()
        or tenant.devices.filter(is_active=True).order_by('-last_seen').first()
    )
    if device and device.node_addr:
        host = device.node_addr.split(':')[0]
        return f"http://{host}:5050"
    return ""


def build_mon_espace_context(tenant) -> dict:
    """Contexte de l'espace PME (`GET /api/tenants/me/`)."""
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    devices = tenant.devices.filter(is_active=True).order_by('-last_seen')
    pgadmin = _derive_credentials(
        str(tenant.registration_token), tenant.name
    )

    return {
        'tenant': tenant,
        'active_license': active_license,
        'devices': devices,
        'pgadmin': pgadmin,
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


def _node_image(request) -> str:
    """Référence complète de l'image du nœud, marqueur `{registry}` résolu."""
    modele = getattr(settings, 'SS_NODE_IMAGE', '{registry}/ss-node:v2')
    return modele.replace('{registry}', _registry_host(request))


def _mode_noeud_par_defaut(tenant: Tenant) -> str:
    """« active » si ce tenant n'a pas encore de primaire, « passive » sinon.

    Même raisonnement que l'installeur Linux, qui interroge
    `/api/devices/cluster-status/` avant de choisir : le premier nœud d'un
    cluster est le primaire, les suivants sont des standbys. Le décider ici
    évite d'avoir à faire éditer un champ de plus à la PME — et deux nœuds
    partis en « active » sur le même cluster, c'est un split-brain.
    """
    a_deja_un_primaire = tenant.devices.filter(
        is_active=True, node_role='primary',
    ).exists()
    return 'passive' if a_deja_un_primaire else 'active'


def build_compose_response(request, tenant: Tenant) -> HttpResponse:
    """Génère la réponse docker-compose.yml pré-configurée pour ce tenant.

    Appelé par `GET /api/tenants/me/download/compose/` et par la voie PME
    `GET /api/tenants/pme/compose/`.
    """
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    node_image = _node_image(request)
    node_mode = _mode_noeud_par_defaut(tenant)
    saas_host, saas_port = _saas_host_port(request)

    content = render(request, 'tenants/docker-compose.yml.tmpl', {
        'tenant': tenant,
        'relay_url': relay_url,
        'node_image': node_image,
        'node_mode': node_mode,
        'saas_host': saas_host,
        'saas_port': saas_port,
    }).content.decode('utf-8')

    response = HttpResponse(content, content_type='application/x-yaml; charset=utf-8')
    response['Content-Disposition'] = 'attachment; filename="docker-compose.yml"'
    return response


def build_installeur_response(request, tenant: Tenant, os_type: str) -> HttpResponse:
    """
    Génère la réponse installateur PME pour Windows (.bat) ou Linux (.sh).
    L'installateur inclut l'installation automatique de Docker et la configuration
    du nœud pré-remplie. Les mises à jour sont gérées automatiquement par Watchtower.

    Appelé par `GET /api/tenants/me/download/installer/<os>/`. `os_type` doit
    valoir `'windows'` ou `'linux'` — à valider par l'appelant.
    """
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    node_image = _node_image(request)
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
        'node_mode': _mode_noeud_par_defaut(tenant),
        'relay_url': relay_url,
        'node_image': node_image,
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

    if os_type == 'windows':
        content = render(request, 'tenants/install-windows.bat.tmpl', ctx).content.decode('utf-8')
        response = HttpResponse(content, content_type='application/octet-stream')
        response['Content-Disposition'] = f'attachment; filename="install-{slugify(tenant.name)}.bat"'
    else:
        content = render(request, 'tenants/install-linux.sh.tmpl', ctx).content.decode('utf-8')
        response = HttpResponse(content, content_type='application/octet-stream')
        response['Content-Disposition'] = f'attachment; filename="install-{slugify(tenant.name)}.sh"'

    return response
