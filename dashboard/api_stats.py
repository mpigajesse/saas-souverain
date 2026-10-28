"""`GET /api/dashboard/stats/` — voir CONTRAT-API.md section 2.

Réservé aux comptes éditeur (staff). Compteurs calculés par `dashboard.services`.
"""
from rest_framework.decorators import api_view, permission_classes
from rest_framework.response import Response

from api.permissions import IsStaffUser
from tenants.serializers import TenantSerializer

from .services import build_dashboard_context


@api_view(['GET'])
@permission_classes([IsStaffUser])
def dashboard_stats(request):
    ctx = build_dashboard_context()
    return Response({
        'stats': ctx['stats'],
        'recent_tenants': TenantSerializer(ctx['recent_tenants'], many=True).data,
    })
