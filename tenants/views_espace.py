from django.contrib.auth.decorators import login_required
from django.shortcuts import render, redirect

from licenses.models import License
from devices.models import Device


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

    return render(request, 'tenants/mon-espace.html', {
        'tenant': tenant,
        'active_license': active_license,
        'devices': devices,
    })
