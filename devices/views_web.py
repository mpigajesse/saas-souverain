from datetime import timedelta

from django.contrib import messages
from django.contrib.auth.decorators import login_required
from django.shortcuts import get_object_or_404, redirect, render
from django.utils import timezone

from tenants.models import Tenant
from .models import Device

ONLINE_THRESHOLD = timedelta(minutes=2)


@login_required
def device_list(request):
    devices = Device.objects.select_related('tenant').order_by('-last_seen')
    return render(request, 'devices/list.html', {'devices': devices})


@login_required
def cluster_overview(request):
    """Supervision des clusters PME — vue par tenant avec santé et licence."""
    now = timezone.now()
    threshold = now - ONLINE_THRESHOLD

    tenants = (
        Tenant.objects
        .prefetch_related('devices', 'licenses')
        .filter(is_active=True)
        .order_by('name')
    )

    clusters = []
    alerts_total = 0

    for tenant in tenants:
        devices = tenant.devices.filter(is_active=True)
        primary = devices.filter(node_role='primary').first()
        standbys = list(devices.filter(node_role='standby'))
        license_status = tenant.licence_status

        # Enrichir chaque nœud avec son statut online
        def enrich(d):
            return {
                'device': d,
                'online': d.last_seen >= threshold if d.last_seen else False,
            }

        primary_info = enrich(primary) if primary else None
        standbys_info = [enrich(s) for s in standbys]

        # Santé globale du cluster
        total = devices.count()
        online_count = devices.filter(last_seen__gte=threshold).count()

        if total == 0:
            health = 'empty'
        elif primary is None:
            health = 'no_primary'
        elif not (primary_info and primary_info['online']):
            health = 'primary_offline'
        elif len(standbys) == 0:
            health = 'no_standby'
        elif not license_status['ok']:
            health = 'over_license'
        else:
            health = 'ok'

        is_alert = health not in ('ok', 'empty')
        if is_alert:
            alerts_total += 1

        clusters.append({
            'tenant': tenant,
            'primary': primary_info,
            'standbys': standbys_info,
            'total': total,
            'online_count': online_count,
            'license': license_status,
            'health': health,
            'is_alert': is_alert,
        })

    # Trier : alertes en premier
    clusters.sort(key=lambda c: (0 if c['is_alert'] else 1, c['tenant'].name))

    return render(request, 'devices/clusters.html', {
        'clusters': clusters,
        'alerts_total': alerts_total,
        'total_tenants': len(clusters),
    })


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
