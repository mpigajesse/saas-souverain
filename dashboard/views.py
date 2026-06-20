from django.contrib.auth.decorators import login_required
from django.shortcuts import redirect, render
from django.utils import timezone

from tenants.models import Tenant
from licenses.models import License
from devices.models import Device
from devices.views_web import ONLINE_THRESHOLD, compute_cluster_snapshot


@login_required
def dashboard(request):
    # Les utilisateurs PME (non-staff) sont redirigés vers leur espace
    if not request.user.is_staff and hasattr(request.user, 'tenant'):
        return redirect('bienvenue')

    threshold = timezone.now() - ONLINE_THRESHOLD

    all_tenants = list(
        Tenant.objects.prefetch_related('licenses', 'devices', 'agent_verdicts').all()
    )
    active_tenants = [t for t in all_tenants if t.is_active]
    tenants_over_limit = sum(1 for t in all_tenants if not t.licence_status['ok'])

    # ── Flotte de nœuds ──────────────────────────────────────────────────────
    total_devices = Device.objects.filter(is_active=True).count()
    devices_online = Device.objects.filter(is_active=True, last_seen__gte=threshold).count()
    online_pct = round(devices_online / total_devices * 100) if total_devices else 0

    # ── Santé des clusters + réplication ─────────────────────────────────────
    clusters_ok = clusters_alert = clusters_empty = 0
    replication_active = 0       # clusters avec au moins un standby en streaming
    clusters_with_standby = 0
    for tenant in active_tenants:
        snap = compute_cluster_snapshot(tenant, threshold)
        if snap['health'] == 'empty':
            clusters_empty += 1
            continue
        if snap['is_alert']:
            clusters_alert += 1
        else:
            clusters_ok += 1
        if snap['standbys']:
            clusters_with_standby += 1
            if snap['streaming'] > 0:
                replication_active += 1
    clusters_total = clusters_ok + clusters_alert

    # ── Occupation des licences (sièges) ─────────────────────────────────────
    seats_used = seats_total = 0
    for tenant in active_tenants:
        ls = tenant.licence_status
        seats_used += ls.get('used', 0) or 0
        seats_total += ls.get('total', 0) or 0
    seats_pct = round(seats_used / seats_total * 100) if seats_total else 0

    # ── Agent IA — dernier verdict par cluster ───────────────────────────────
    risk_counts = {'critique': 0, 'surveiller': 0, 'sain': 0}
    analyzed = 0
    for tenant in active_tenants:
        v = tenant.agent_verdicts.first()  # ordering = -created_at
        if v:
            analyzed += 1
            risk_counts[v.risk_level] = risk_counts.get(v.risk_level, 0) + 1

    # ── Serveur éditeur (hôte) ───────────────────────────────────────────────
    from devices.host_monitor import collect_host_metrics, host_health
    host = collect_host_metrics()
    host['health'] = host_health(host)

    stats = {
        'total_tenants': len(all_tenants),
        'active_licenses': License.objects.filter(is_active=True).count(),
        'total_devices': total_devices,
        'tenants_over_limit': tenants_over_limit,
        # KPI enrichis
        'devices_online': devices_online,
        'online_pct': online_pct,
        'clusters_total': clusters_total,
        'clusters_ok': clusters_ok,
        'clusters_alert': clusters_alert,
        'replication_active': replication_active,
        'clusters_with_standby': clusters_with_standby,
        'seats_used': seats_used,
        'seats_total': seats_total,
        'seats_pct': seats_pct,
        'risk_counts': risk_counts,
        'analyzed': analyzed,
        'host': host,
    }
    recent_tenants = Tenant.objects.order_by('-created_at')[:5]
    return render(request, 'dashboard/index.html', {
        'stats': stats,
        'recent_tenants': recent_tenants,
    })
