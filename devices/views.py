from rest_framework import viewsets
from rest_framework.decorators import action
from rest_framework.response import Response

from api.permissions import IsStaffUser

from .models import Device
from .serializers import DeviceSerializer


class DeviceViewSet(viewsets.ModelViewSet):
    """Parc installé — `/api/devices/` — voir CONTRAT-API.md section 2.

    Réservé aux comptes éditeur (staff) : le parc de tous les tenants y est
    visible. L'annonce d'un nœud (`/api/devices/register/`) et le statut cluster
    (`/api/devices/cluster-status/`) restent publics — ils sont appelés par les
    nœuds PME eux-mêmes, authentifiés par jeton d'enregistrement.
    """

    permission_classes = [IsStaffUser]
    queryset = Device.objects.select_related('tenant').all()
    serializer_class = DeviceSerializer

    @action(detail=True, methods=['post'], permission_classes=[IsStaffUser])
    def toggle(self, request, pk=None):
        """`POST /api/devices/<uuid>/toggle/` — bascule `is_active`.

        Réservé aux comptes éditeur (staff) — voir CONTRAT-API.md section 2.
        """
        device = self.get_object()
        device.is_active = not device.is_active
        device.save(update_fields=['is_active'])
        return Response({'id': str(device.id), 'is_active': device.is_active})
