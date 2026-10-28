from rest_framework import viewsets

from api.permissions import IsStaffUser

from .models import License
from .serializers import LicenseSerializer


class LicenseViewSet(viewsets.ModelViewSet):
    """CRUD des licences — `/api/licenses/` — voir CONTRAT-API.md section 2.

    Réservé aux comptes éditeur (staff) : la délivrance et la révocation d'une
    licence sont des actes de l'éditeur. Un compte PME lit la sienne dans
    `GET /api/tenants/me/`.
    """

    permission_classes = [IsStaffUser]
    queryset = License.objects.select_related('tenant').all()
    serializer_class = LicenseSerializer
