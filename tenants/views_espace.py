import hashlib

from django.contrib.auth.decorators import login_required
from django.shortcuts import render, redirect
from django.utils.text import slugify

from licenses.models import License
from devices.models import Device


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
    """Contexte de l'espace PME. Partagé par la vue web `mon_espace` et
    `GET /api/tenants/me/`.
    """
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


@login_required
def mon_espace(request):
    user = request.user
    if user.is_staff:
        return redirect('dashboard')

    if not hasattr(user, 'tenant'):
        return redirect('bienvenue')

    return render(request, 'tenants/mon-espace.html', build_mon_espace_context(user.tenant))
