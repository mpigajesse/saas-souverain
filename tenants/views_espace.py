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


@login_required
def mon_espace(request):
    user = request.user
    if user.is_staff:
        return redirect('dashboard')

    if not hasattr(user, 'tenant'):
        return redirect('bienvenue')

    tenant = user.tenant
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    devices = tenant.devices.filter(is_active=True).order_by('-last_seen')
    pgadmin = _derive_credentials(
        str(tenant.registration_token), tenant.name
    )

    return render(request, 'tenants/mon-espace.html', {
        'tenant': tenant,
        'active_license': active_license,
        'devices': devices,
        'pgadmin': pgadmin,
    })
