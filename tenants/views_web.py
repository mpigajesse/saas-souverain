import io
import uuid
from datetime import date, timedelta

from django.conf import settings
from django.utils.text import slugify
from django.contrib import messages
from django.contrib.auth import login
from django.contrib.auth.decorators import login_required
from django.contrib.auth.models import User
from django.http import HttpResponse
from django.shortcuts import get_object_or_404, redirect, render
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response

from licenses.models import License
from devices.models import Device
from .forms import InscriptionForm, TenantForm
from .models import Tenant


@login_required
def tenant_list(request):
    tenants = Tenant.objects.order_by('-created_at')
    return render(request, 'tenants/list.html', {'tenants': tenants})


@login_required
def tenant_detail(request, pk):
    tenant = get_object_or_404(Tenant, pk=pk)
    licenses = License.objects.filter(tenant=tenant).order_by('-created_at')
    devices = Device.objects.filter(tenant=tenant).order_by('-last_seen')
    return render(request, 'tenants/detail.html', {
        'tenant': tenant,
        'licenses': licenses,
        'devices': devices,
    })


@login_required
def tenant_create(request):
    if request.method == 'POST':
        form = TenantForm(request.POST)
        if form.is_valid():
            tenant = form.save()
            messages.success(request, f'Tenant « {tenant.name} » créé avec succès.')
            return redirect('tenant-detail', pk=tenant.pk)
    else:
        form = TenantForm()
    return render(request, 'tenants/form.html', {'form': form, 'action': 'Créer un tenant'})


@login_required
def tenant_update(request, pk):
    tenant = get_object_or_404(Tenant, pk=pk)
    if request.method == 'POST':
        form = TenantForm(request.POST, instance=tenant)
        if form.is_valid():
            form.save()
            messages.success(request, f'Tenant « {tenant.name} » mis à jour.')
            return redirect('tenant-detail', pk=tenant.pk)
    else:
        form = TenantForm(instance=tenant)
    return render(request, 'tenants/form.html', {'form': form, 'tenant': tenant, 'action': 'Modifier le tenant'})


@login_required
def tenant_delete(request, pk):
    tenant = get_object_or_404(Tenant, pk=pk)
    if request.method == 'POST':
        name = tenant.name
        tenant.delete()
        messages.success(request, f'Tenant « {name} » supprimé.')
        return redirect('tenant-list')
    return render(request, 'tenants/confirm_delete.html', {'tenant': tenant})


def _create_tenant_account(cleaned_data: dict) -> tuple[User, Tenant]:
    """Crée le `User` Django, le `Tenant` lié et sa licence d'essai.

    Partagé par la vue web `inscription` et `POST /api/auth/register/` — les
    deux passent par `InscriptionForm` en amont, donc les mêmes validations
    (email unique, mots de passe identiques) s'appliquent dans les deux cas.
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


def inscription(request):
    """Page d'inscription publique — accessible sans login."""
    if request.user.is_authenticated:
        if hasattr(request.user, 'tenant'):
            return redirect('bienvenue')
        return redirect('dashboard')

    if request.method == 'POST':
        form = InscriptionForm(request.POST)
        if form.is_valid():
            user, tenant = _create_tenant_account(form.cleaned_data)
            # Connecter automatiquement
            login(request, user)
            messages.success(request, f'Bienvenue chez EL BARAA CONSULT, {tenant.name} !')
            return redirect('bienvenue')
    else:
        form = InscriptionForm()

    return render(request, 'tenants/inscription.html', {'form': form})


def build_bienvenue_context(tenant: Tenant) -> dict:
    """Contexte de la page d'accueil post-inscription.

    Partagé par la vue web `bienvenue` et `GET /api/tenants/me/onboarding/`.
    """
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    active_devices = tenant.devices.filter(is_active=True).count()
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    return {
        'tenant': tenant,
        'active_license': active_license,
        'active_devices': active_devices,
        'relay_url': relay_url,
    }


@login_required
def bienvenue(request):
    """Page d'accueil post-inscription — guide l'installation du cluster Docker."""
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')

    return render(request, 'tenants/bienvenue.html', build_bienvenue_context(tenant))


def _saas_host_port(request) -> tuple[str, str]:
    """Dérive host/port SaaS à partir de la requête.

    L'installeur sera téléchargé depuis la VM PME, donc `request.get_host()`
    retourne l'IP LAN du PC physique — logique inchangée, juste extraite pour
    être appelée à la fois par les vues web et par l'API.
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

    Partagé par la vue web `telecharger_compose` et
    `GET /api/tenants/me/download/compose/`.
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


@login_required
def telecharger_compose(request):
    """Génère et télécharge un docker-compose.yml pré-configuré pour ce tenant."""
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')

    return build_compose_response(request, tenant)


def build_installeur_response(request, tenant: Tenant, os_type: str) -> HttpResponse:
    """
    Génère la réponse installateur PME pour Windows (.bat) ou Linux (.sh).
    L'installateur inclut l'installation automatique de Docker et la configuration
    du nœud pré-remplie. Les mises à jour sont gérées automatiquement par Watchtower.

    Partagé par la vue web `telecharger_installeur` et
    `GET /api/tenants/me/download/installer/<os>/`. `os_type` doit valoir
    `'windows'` ou `'linux'` — à valider par l'appelant.
    """
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    node_image = _node_image(request)
    registry_host = _registry_host(request)
    saas_host, saas_port = _saas_host_port(request)

    from datetime import datetime
    import unicodedata

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


@login_required
def telecharger_installeur(request, os_type):
    """
    Vue web : télécharge l'installateur PME pour Windows (.bat) ou Linux (.sh).
    """
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')
    if os_type not in ('windows', 'linux'):
        return redirect('bienvenue')

    return build_installeur_response(request, tenant, os_type)


@api_view(['GET'])
@permission_classes([AllowAny])
def compose_telecharger_pme(request):
    """`GET /api/tenants/pme/compose/?token=<registration_token>`.

    Voie de repli des installeurs PME, calquée sur
    `dashboard.api_images.node_image_telecharger_pme` : la machine qui déploie
    le nœud n'a aucune session éditeur, elle ne peut donc pas passer par
    `/api/tenants/me/download/compose/`. Elle s'authentifie avec le seul secret
    dont elle dispose — le jeton d'enregistrement du tenant — exactement comme
    `devices/api_cluster_status.py`.

    C'est ce point d'entrée qui fait de `templates/tenants/docker-compose.yml.tmpl`
    la définition UNIQUE de la pile : les deux installeurs la téléchargent au
    lieu de la régénérer chacun à sa façon, donc Windows et Linux déploient
    rigoureusement les mêmes services.
    """
    token_raw = request.GET.get('token', '').strip()
    if not token_raw:
        return Response({'detail': 'Paramètre token manquant.'}, status=400)

    try:
        uuid.UUID(token_raw)
    except ValueError:
        return Response({'detail': 'Format UUID invalide.'}, status=400)

    try:
        tenant = Tenant.objects.get(registration_token=token_raw, is_active=True)
    except Tenant.DoesNotExist:
        return Response({'detail': 'Token invalide ou tenant inactif.'}, status=403)

    return build_compose_response(request, tenant)
