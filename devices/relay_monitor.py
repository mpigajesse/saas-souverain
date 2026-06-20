"""
Supervision du serveur relais zero-knowledge.

L'agent vérifie la SANTÉ du relais (disponibilité, uptime, stockage) à partir
de son endpoint /health, qui ne renvoie que des MÉTADONNÉES :
disponibilité, version, uptime, nombre de tenants ayant des blobs, réseaux.

⚠️ Souveraineté totale : l'agent n'accède JAMAIS au contenu des blobs chiffrés
répliqués sur le relais. Il ne lit que des compteurs et l'état du service.
"""
from __future__ import annotations

import json
import urllib.error
import urllib.request

from django.conf import settings
from django.utils import timezone


def _format_uptime_h(secs: int) -> float:
    return round((secs or 0) / 3600, 1)


def collect_relay_metrics() -> dict:
    """Interroge le /health du relais. Aucun accès aux blobs."""
    relay_url = getattr(settings, "RELAY_URL", "http://localhost:8080").rstrip("/")

    metrics = {
        'reachable': False,
        'status': 'unreachable',
        'version': '—',
        'hostname': '—',
        'uptime_h': 0.0,
        'blob_tenants': 0,        # NOMBRE de tenants ayant des blobs (métadonnée)
        'networks_count': 0,
        'zero_knowledge': True,
        'relay_url': relay_url,
        'captured_at': timezone.now().isoformat(timespec='seconds'),
    }
    try:
        with urllib.request.urlopen(f"{relay_url}/health", timeout=5) as resp:
            data = json.loads(resp.read())
        metrics.update({
            'reachable': True,
            'status': data.get('status', 'ok'),
            'version': data.get('version', '—'),
            'hostname': data.get('hostname', '—'),
            'uptime_h': _format_uptime_h(data.get('uptime_secs', 0)),
            'blob_tenants': data.get('blob_tenants', 0),
            'networks_count': len(data.get('networks', []) or []),
            'zero_knowledge': bool(data.get('zero_knowledge', True)),
        })
    except Exception as exc:
        metrics['error'] = str(exc)
    return metrics


def relay_health(metrics: dict) -> str:
    """Niveau de santé déterministe du relais (sans IA)."""
    if not metrics.get('reachable', False):
        return 'critique'
    # Le relais DOIT rester zero-knowledge : une réponse l'indiquant à false
    # serait une anomalie de sécurité majeure.
    if not metrics.get('zero_knowledge', True):
        return 'critique'
    return 'sain'
