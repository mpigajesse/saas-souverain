from rest_framework import viewsets
from .models import License
from .serializers import LicenseSerializer


class LicenseViewSet(viewsets.ModelViewSet):
    queryset = License.objects.select_related('tenant').all()
    serializer_class = LicenseSerializer
