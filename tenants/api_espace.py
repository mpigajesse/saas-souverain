"""Espace PME — `/api/tenants/me/...` — voir CONTRAT-API.md section 3.

Toutes les vues sont réservées aux comptes non-staff rattachés à un tenant
(`api.permissions.IsTenantUser` → 403 sinon). La logique métier vit dans
`tenants.services` : on ne la réécrit pas, on l'appelle.
"""
from django.conf import settings
from rest_framework.decorators import api_view, permission_classes
from rest_framework.response import Response

from api.permissions import IsTenantUser
from devices.serializers import DeviceSerializer
from licenses.serializers import LicenseSerializer

from .serializers import TenantSerializer
from .services import (
    _pgadmin_url,
    build_bienvenue_context,
    build_compose_response,
    build_installeur_response,
    build_mon_espace_context,
)


@api_view(['GET'])
@permission_classes([IsTenantUser])
def tenant_me(request):
    tenant = request.user.tenant
    ctx = build_mon_espace_context(tenant)
    active_license = ctx['active_license']
    devices = ctx['devices']

    return Response({
        'tenant': TenantSerializer(tenant).data,
        'active_license': LicenseSerializer(active_license).data if active_license else None,
        'devices': DeviceSerializer(devices, many=True).data,
        'active_devices': devices.count(),
        'pgadmin': {
            'url': _pgadmin_url(tenant),
            'email': ctx['pgadmin']['pgadmin_email'],
            'password': ctx['pgadmin']['pgadmin_password'],
        },
        'relay_url': getattr(settings, 'RELAY_URL', 'http://localhost:8080'),
    })


@api_view(['GET'])
@permission_classes([IsTenantUser])
def tenant_onboarding(request):
    tenant = request.user.tenant
    ctx = build_bienvenue_context(tenant)
    active_license = ctx['active_license']

    return Response({
        'tenant': TenantSerializer(tenant).data,
        'active_license': LicenseSerializer(active_license).data if active_license else None,
        'active_devices': ctx['active_devices'],
        'relay_url': ctx['relay_url'],
        'downloads': {
            'compose': '/api/tenants/me/download/compose/',
            'installer_linux': '/api/tenants/me/download/installer/linux/',
            'installer_windows': '/api/tenants/me/download/installer/windows/',
        },
    })


@api_view(['GET'])
@permission_classes([IsTenantUser])
def download_compose(request):
    return build_compose_response(request, request.user.tenant)


@api_view(['GET'])
@permission_classes([IsTenantUser])
def download_installer(request, os_type):
    if os_type not in ('windows', 'linux'):
        return Response({'detail': "os doit être 'linux' ou 'windows'."}, status=400)
    return build_installeur_response(request, request.user.tenant, os_type)
