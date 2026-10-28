"""`GET /api/relay/status/` — voir CONTRAT-API.md section 2.

Réservé aux comptes éditeur (staff). État collecté par
`dashboard.relay.build_relay_status`. Le relais injoignable reste un état
normal (`health.reachable: False`), jamais une 500.
"""
from rest_framework.decorators import api_view, permission_classes
from rest_framework.response import Response

from api.permissions import IsStaffUser

from .relay import build_relay_status


@api_view(['GET'])
@permission_classes([IsStaffUser])
def relay_status_view(request):
    data = build_relay_status()

    tenants = [
        {
            'tenant': {'id': str(item['tenant'].id), 'name': item['tenant'].name},
            'nodes': item['nodes'],
        }
        for item in data['tenants_data']
    ]

    return Response({
        'relay_url': data['relay_url'],
        'health': data['health'],
        'total_nodes': data['total_nodes'],
        'tenants': tenants,
        'editeur_subnet': data['editeur_subnet'],
        'relay_subnet': data['relay_subnet'],
        'pme_subnet': data['pme_subnet'],
    })
