import uuid

from django.utils import timezone
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response
from rest_framework import status

from tenants.models import Tenant
from .models import Device


@api_view(['POST'])
@permission_classes([AllowAny])
def device_register(request):
    """
    Appelé par le binaire PME au premier lancement et à chaque démarrage.
    Corps JSON attendu :
      - tenant_token (UUID)          : token d'enregistrement du tenant
      - installation_id (UUID)       : UUID généré une fois par installation
      - hostname (str, optionnel)    : nom de la machine
      - os (str, optionnel)          : ex. "Windows 11"
      - mac_address (str, optionnel) : indicatif seulement
    """
    tenant_token_raw = request.data.get('tenant_token')
    installation_id_raw = request.data.get('installation_id')

    # Validation des champs obligatoires
    if not tenant_token_raw or not installation_id_raw:
        return Response(
            {'status': 'error', 'message': 'tenant_token et installation_id sont obligatoires.'},
            status=status.HTTP_400_BAD_REQUEST,
        )

    # Validation des UUIDs
    try:
        tenant_token = uuid.UUID(str(tenant_token_raw))
        installation_id = uuid.UUID(str(installation_id_raw))
    except ValueError:
        return Response(
            {'status': 'error', 'message': 'tenant_token ou installation_id invalide (format UUID attendu).'},
            status=status.HTTP_400_BAD_REQUEST,
        )

    # Vérifier le tenant
    try:
        tenant = Tenant.objects.get(registration_token=tenant_token, is_active=True)
    except Tenant.DoesNotExist:
        return Response(
            {'status': 'refused', 'reason': 'invalid_token', 'message': 'Token invalide ou tenant inactif.'},
            status=status.HTTP_403_FORBIDDEN,
        )

    # Vérifier la licence active
    active_license = tenant.licenses.filter(is_active=True).order_by('-created_at').first()
    if active_license is None:
        return Response(
            {'status': 'refused', 'reason': 'no_license', 'message': 'Aucune licence active pour ce tenant.'},
            status=status.HTTP_403_FORBIDDEN,
        )

    # Créer ou retrouver la machine par installation_id
    node_addr = request.data.get('node_addr', '')
    web_addr = request.data.get('web_addr', '')
    node_role = request.data.get('node_role', '')
    device, created = Device.objects.get_or_create(
        installation_id=installation_id,
        defaults={
            'tenant': tenant,
            'hostname': request.data.get('hostname', ''),
            'os': request.data.get('os', ''),
            'mac_address': request.data.get('mac_address', ''),
            'node_addr': node_addr,
            'web_addr': web_addr,
            'node_role': node_role,
            'is_active': True,
        },
    )

    # Si la machine existait déjà, mettre à jour les champs et last_seen (auto_now)
    if not created:
        device.hostname = request.data.get('hostname', device.hostname)
        device.os = request.data.get('os', device.os)
        if node_addr:
            device.node_addr = node_addr
        if web_addr:
            device.web_addr = web_addr
        if node_role:
            # Failover automatique détecté : le nœud était standby et annonce maintenant primary
            if device.node_role == 'standby' and node_role == 'primary':
                device.failover_count += 1
                device.last_failover_at = timezone.now()
            device.node_role = node_role
        device.save()

        if not device.is_active:
            return Response(
                {
                    'status': 'refused',
                    'reason': 'device_disabled',
                    'message': "Cette machine a été désactivée par l'administrateur.",
                },
                status=status.HTTP_403_FORBIDDEN,
            )

    # Vérifier les sièges disponibles (après get_or_create pour inclure la machine actuelle)
    active_device_count = tenant.devices.filter(is_active=True).count()
    if active_device_count > active_license.seats:
        # Nouvelle machine qui dépasse le quota → désactiver et refuser
        if created:
            device.is_active = False
            device.save()
        return Response(
            {
                'status': 'refused',
                'reason': 'licence_exceeded',
                'message': f'Nombre maximum de postes atteint ({active_license.seats}/{active_license.seats}).',
                'seats_used': active_device_count - 1,
                'seats_total': active_license.seats,
            },
            status=status.HTTP_403_FORBIDDEN,
        )

    return Response(
        {
            'status': 'authorized',
            'device_id': str(device.id),
            'tenant_name': tenant.name,
            'message': 'Machine enregistrée et autorisée.' if created else 'Machine reconnue et autorisée.',
            'seats_used': active_device_count,
            'seats_total': active_license.seats,
        },
        status=status.HTTP_201_CREATED if created else status.HTTP_200_OK,
    )
