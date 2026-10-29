"""Espace PME — `/api/tenants/me/...` — voir CONTRAT-API.md section 3.

Toutes les vues sont réservées aux comptes non-staff rattachés à un tenant
(`api.permissions.IsTenantUser` → 403 sinon). La logique métier vit dans
`tenants.services` : on ne la réécrit pas, on l'appelle.
"""
from django.conf import settings
from django.utils.text import slugify
from rest_framework.decorators import api_view, permission_classes
from rest_framework.response import Response

from api.permissions import IsTenantUser
from devices.serializers import DeviceSerializer
from licenses.serializers import LicenseSerializer

from .pile import PileNonPubliee
from .serializers import TenantSerializer
from .services import (
    TYPES_MACHINE,
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
        # Le mot de passe pgAdmin n'est PAS connu du portail : il est généré
        # sur le réseau de la PME et affiché en fin d'installation (ADR-005).
        'pgadmin': {
            'url': _pgadmin_url(tenant),
            'email': f'admin@pme{slugify(tenant.name)}.com',
        },
        'besoin_temoin': ctx['besoin_temoin'],
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
            'installer_temoin_linux': '/api/tenants/me/download/installer/linux/?type=temoin',
            'installer_temoin_windows': '/api/tenants/me/download/installer/windows/?type=temoin',
        },
    })


def type_machine_demande(request):
    """Type de machine demandé (`?type=serveur|temoin`, serveur par défaut),
    ou None s'il est invalide."""
    type_machine = request.GET.get('type', 'serveur')
    return type_machine if type_machine in TYPES_MACHINE else None


def reponse_pile_non_publiee(exc):
    return Response(
        {'detail': f"Images de la pile non publiées par l'éditeur : {exc}"},
        status=503,
    )


@api_view(['GET'])
@permission_classes([IsTenantUser])
def download_compose(request):
    type_machine = type_machine_demande(request)
    if type_machine is None:
        return Response({'detail': "type doit être 'serveur' ou 'temoin'."}, status=400)
    try:
        return build_compose_response(request, request.user.tenant, type_machine)
    except PileNonPubliee as exc:
        return reponse_pile_non_publiee(exc)


@api_view(['GET'])
@permission_classes([IsTenantUser])
def download_installer(request, os_type):
    if os_type not in ('windows', 'linux'):
        return Response({'detail': "os doit être 'linux' ou 'windows'."}, status=400)
    type_machine = type_machine_demande(request)
    if type_machine is None:
        return Response({'detail': "type doit être 'serveur' ou 'temoin'."}, status=400)
    return build_installeur_response(request, request.user.tenant, os_type, type_machine)
