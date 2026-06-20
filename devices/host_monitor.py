"""
Supervision du serveur éditeur SaaS lui-même (la machine qui héberge Django).

Collecte des métriques système (CPU, RAM, disque, uptime) via psutil, plus
quelques métriques applicatives (parc, base). Aucune donnée métier : ce sont
des métriques d'exploitation de l'infrastructure éditeur.
"""
from __future__ import annotations

import time

import psutil
from django.conf import settings
from django.db import connection
from django.utils import timezone


def _safe(fn, default=None):
    try:
        return fn()
    except Exception:
        return default


def collect_host_metrics() -> dict:
    """Métriques système + applicatives du serveur éditeur."""
    mem = psutil.virtual_memory()
    disk = psutil.disk_usage(str(settings.BASE_DIR))
    uptime_s = time.time() - psutil.boot_time()

    # Métriques applicatives (légères) — comptage du parc et test base.
    from .models import Device
    from tenants.models import Tenant

    db_ok = True
    try:
        with connection.cursor() as cur:
            cur.execute("SELECT 1")
            cur.fetchone()
    except Exception:
        db_ok = False

    online_threshold = timezone.now() - timezone.timedelta(minutes=2)

    return {
        # Système
        'cpu_percent': psutil.cpu_percent(interval=0.3),
        'cpu_cores': psutil.cpu_count(logical=True) or 0,
        'mem_percent': mem.percent,
        'mem_used_gb': round(mem.used / 1e9, 1),
        'mem_total_gb': round(mem.total / 1e9, 1),
        'disk_percent': disk.percent,
        'disk_used_gb': round(disk.used / 1e9, 1),
        'disk_total_gb': round(disk.total / 1e9, 1),
        'uptime_h': round(uptime_s / 3600, 1),
        'process_count': len(_safe(psutil.pids, []) or []),
        # Applicatif (éditeur)
        'db_ok': db_ok,
        'tenants_actifs': _safe(lambda: Tenant.objects.filter(is_active=True).count(), 0),
        'devices_actifs': _safe(lambda: Device.objects.filter(is_active=True).count(), 0),
        'devices_en_ligne': _safe(
            lambda: Device.objects.filter(is_active=True, last_seen__gte=online_threshold).count(), 0
        ),
        'captured_at': timezone.now().isoformat(timespec='seconds'),
    }


def host_health(metrics: dict) -> str:
    """Niveau de santé déterministe du serveur éditeur (sans IA)."""
    if not metrics.get('db_ok', True):
        return 'critique'
    if metrics.get('cpu_percent', 0) >= 90 or metrics.get('mem_percent', 0) >= 92 \
            or metrics.get('disk_percent', 0) >= 95:
        return 'critique'
    if metrics.get('cpu_percent', 0) >= 75 or metrics.get('mem_percent', 0) >= 80 \
            or metrics.get('disk_percent', 0) >= 85:
        return 'surveiller'
    return 'sain'
