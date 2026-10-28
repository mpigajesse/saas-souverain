from rest_framework import viewsets

from api.permissions import IsStaffUser

from .models import Tenant
from .serializers import TenantSerializer


class TenantViewSet(viewsets.ModelViewSet):
    """CRUD des tenants — `/api/tenants/` — voir CONTRAT-API.md section 2.

    Réservé aux comptes éditeur (staff). Sans cette permission explicite, le
    `IsAuthenticated` par défaut de REST_FRAMEWORK laisserait un compte PME
    — que n'importe qui peut créer via `POST /api/auth/register/` — lister et
    supprimer les tenants des autres. L'espace PME passe par `/api/tenants/me/`.
    """

    permission_classes = [IsStaffUser]
    queryset = Tenant.objects.all()
    serializer_class = TenantSerializer
