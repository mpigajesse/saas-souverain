"""`GET /api/clusters/` — voir CONTRAT-API.md section 2.

Réservé aux comptes éditeur (staff). Vue calculée par
`devices.services.build_cluster_overview` (seuil en ligne : 2 minutes).
"""
from rest_framework.decorators import api_view, permission_classes
from rest_framework.response import Response

from api.permissions import IsStaffUser

from .serializers import DeviceSerializer
from .services import build_cluster_overview


def _serialize_node(node_info):
    if node_info is None:
        return None
    return {
        'device': DeviceSerializer(node_info['device']).data,
        'online': node_info['online'],
    }


@api_view(['GET'])
@permission_classes([IsStaffUser])
def clusters_view(request):
    data = build_cluster_overview()

    clusters = [
        {
            'tenant': {
                'id': str(c['tenant'].id),
                'name': c['tenant'].name,
                'email': c['tenant'].email,
            },
            'primary': _serialize_node(c['primary']),
            'standbys': [_serialize_node(s) for s in c['standbys']],
            'total': c['total'],
            'online_count': c['online_count'],
            'license': c['license'],
            'health': c['health'],
            'is_alert': c['is_alert'],
        }
        for c in data['clusters']
    ]

    return Response({
        'alerts_total': data['alerts_total'],
        'total_tenants': data['total_tenants'],
        'clusters': clusters,
    })
