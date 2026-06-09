from django.contrib import messages
from django.contrib.auth.decorators import login_required
from django.shortcuts import get_object_or_404, redirect, render

from .models import Device


@login_required
def device_list(request):
    devices = Device.objects.select_related('tenant').order_by('-last_seen')
    return render(request, 'devices/list.html', {'devices': devices})


@login_required
def device_toggle(request, pk):
    """Activer ou désactiver une machine du parc."""
    device = get_object_or_404(Device, pk=pk)
    if request.method == 'POST':
        device.is_active = not device.is_active
        device.save(update_fields=['is_active'])
        etat = 'activée' if device.is_active else 'désactivée'
        if not device.is_active:
            messages.warning(
                request,
                f'Machine « {device.hostname or str(device.installation_id)[:8]} » désactivée. '
                f'⚠ Pensez à effectuer une rotation de la DEK côté cluster PME.'
            )
        else:
            messages.success(request, f'Machine « {device.hostname or str(device.installation_id)[:8]} » réactivée.')
        next_url = request.POST.get('next', 'device-list')
        return redirect(next_url)
    return redirect('device-list')
