import io
from datetime import date, timedelta

from django.conf import settings
from django.utils.text import slugify
from django.contrib import messages
from django.contrib.auth import login
from django.contrib.auth.decorators import login_required
from django.contrib.auth.models import User
from django.http import HttpResponse
from django.shortcuts import get_object_or_404, redirect, render

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


def inscription(request):
    """Page d'inscription publique — accessible sans login."""
    if request.user.is_authenticated:
        if hasattr(request.user, 'tenant'):
            return redirect('bienvenue')
        return redirect('dashboard')

    if request.method == 'POST':
        form = InscriptionForm(request.POST)
        if form.is_valid():
            d = form.cleaned_data

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

            # Connecter automatiquement
            login(request, user)
            messages.success(request, f'Bienvenue chez EL BARAA CONSULT, {tenant.name} !')
            return redirect('bienvenue')
    else:
        form = InscriptionForm()

    return render(request, 'tenants/inscription.html', {'form': form})


@login_required
def bienvenue(request):
    """Page d'accueil post-inscription — guide l'installation du cluster Docker."""
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')

    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    active_devices = tenant.devices.filter(is_active=True).count()
    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')

    return render(request, 'tenants/bienvenue.html', {
        'tenant': tenant,
        'active_license': active_license,
        'active_devices': active_devices,
        'relay_url': relay_url,
    })


@login_required
def telecharger_compose(request):
    """Génère et télécharge un docker-compose.yml pré-configuré pour ce tenant."""
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')

    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    node_image = getattr(settings, 'SS_NODE_IMAGE', 'ss-node:dev')

    host_full = request.get_host()
    if ':' in host_full:
        saas_host, saas_port = host_full.rsplit(':', 1)
    else:
        saas_host = host_full
        saas_port = '443' if request.is_secure() else '80'

    content = render(request, 'tenants/docker-compose.yml.tmpl', {
        'tenant': tenant,
        'relay_url': relay_url,
        'node_image': node_image,
        'saas_host': saas_host,
        'saas_port': saas_port,
    }).content.decode('utf-8')

    response = HttpResponse(content, content_type='application/x-yaml; charset=utf-8')
    response['Content-Disposition'] = 'attachment; filename="docker-compose.yml"'
    return response


@login_required
def telecharger_installeur(request, os_type):
    """
    Génère et télécharge l'installateur PME pour Windows (.bat) ou Linux (.sh).
    L'installateur inclut l'installation automatique de Docker et la configuration
    du nœud pré-remplie. Les mises à jour sont gérées automatiquement par Watchtower.
    """
    tenant = getattr(request.user, 'tenant', None)
    if tenant is None:
        return redirect('dashboard')
    if os_type not in ('windows', 'linux'):
        return redirect('bienvenue')

    relay_url = getattr(settings, 'RELAY_URL', 'http://localhost:8080')
    node_image = getattr(settings, 'SS_NODE_IMAGE', 'ss-node:dev')

    # Dérive host/port à partir de la requête — l'installeur sera téléchargé
    # depuis la VM PME, donc request.get_host() retourne l'IP LAN du PC physique.
    host_full = request.get_host()
    if ':' in host_full:
        saas_host, saas_port = host_full.rsplit(':', 1)
    else:
        saas_host = host_full
        saas_port = '443' if request.is_secure() else '80'

    from datetime import datetime
    ctx = {
        'tenant': tenant,
        'relay_url': relay_url,
        'node_image': node_image,
        'saas_host': saas_host,
        'saas_port': saas_port,
        'generated_at': datetime.now().strftime('%d/%m/%Y à %H:%M'),
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
