"""Supervision du parc installé, appelée par l'API REST (`GET /api/clusters/`)."""
from datetime import timedelta

from django.utils import timezone

from tenants.models import Tenant

ONLINE_THRESHOLD = timedelta(minutes=2)


def build_cluster_overview() -> dict:
    """Supervision des clusters PME — vue par tenant avec santé et licence.

    Calculée pour `GET /api/clusters/`.
    """
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

    return {
        'clusters': clusters,
        'alerts_total': alerts_total,
        'total_tenants': len(clusters),
    }
