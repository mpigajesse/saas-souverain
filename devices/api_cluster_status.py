import uuid

from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response

from tenants.models import Tenant


@api_view(['GET'])
@permission_classes([AllowAny])
def cluster_status(request):
    """
    Renvoie l'état du cluster pour un tenant via son token de licence.
    Utilisé par l'installateur PME pour détecter si ce nœud est primaire ou standby.

    GET /api/devices/cluster-status/?token=<REGISTRATION_TOKEN>

    Réponse :
      {
        "tenant_name": "MPJ",
        "node_count": 1,
        "active_nodes": [
          {"hostname": "kali", "node_addr": "192.168.200.130:9001", "node_role": "primary", "os": "linux"}
        ]
      }
    """
    token_raw = request.GET.get('token', '').strip()
    if not token_raw:
        return Response({'error': 'Paramètre token manquant.'}, status=400)

    try:
        uuid.UUID(token_raw)
    except ValueError:
        return Response({'error': 'Format UUID invalide.'}, status=400)

    try:
        tenant = Tenant.objects.get(registration_token=token_raw, is_active=True)
    except Tenant.DoesNotExist:
        return Response({'error': 'Token invalide ou tenant inactif.'}, status=403)

    devices = list(
        tenant.devices
        .filter(is_active=True)
        .values('node_addr', 'node_role', 'hostname', 'os')
        .order_by('node_role')  # primary avant standby
    )

    return Response({
        'tenant_name': tenant.name,
        'node_count': len(devices),
        'active_nodes': devices,
    })
