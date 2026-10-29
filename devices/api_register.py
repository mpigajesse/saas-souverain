import ipaddress
import uuid

from django.conf import settings
from django.utils import timezone
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response
from rest_framework import status

from tenants.models import Tenant
from .models import Device


def _ip_client(request) -> str:
    """Adresse IP de la machine qui appelle, vue depuis le SaaS.

    Le nœud tourne dans un conteneur : il ne connaît que son IP interne
    (172.x) et n'a aucun moyen fiable de découvrir l'IP LAN de sa machine
    hôte — celle par laquelle les autres nœuds le joindront. Le serveur, lui,
    la voit dans la connexion entrante, car Docker fait sortir le trafic du
    conteneur derrière l'adresse de l'hôte.

    Derrière le proxy nginx, `REMOTE_ADDR` désigne le conteneur nginx : la
    vraie adresse est en tête de `X-Forwarded-For`. Cet en-tête est falsifiable
    par le client, mais sans conséquence ici — le nœud peut de toute façon
    annoncer l'adresse qu'il veut dans `node_addr`.
    """
    transfere = request.META.get('HTTP_X_FORWARDED_FOR', '')
    if transfere:
        candidat = transfere.split(',')[0].strip()
        if candidat:
            return candidat
    return request.META.get('REMOTE_ADDR', '') or ''


def _est_une_ip(valeur: str) -> bool:
    try:
        ipaddress.ip_address(valeur)
        return True
    except ValueError:
        return False


def _ip_exploitable(ip: str) -> bool:
    """Écarte les adresses internes au réseau Docker, inutilisables comme
    adresse d'annonce.

    **Docker Desktop (Windows et macOS) masque l'IP source** : le trafic entrant
    par un port publié est réécrit à l'adresse de la passerelle du réseau bridge
    (172.x.0.1). Toutes les machines du LAN arrivent donc avec la même adresse,
    celle de la passerelle — inexploitable. Sur un hôte Docker Linux, l'IP réelle
    du client est en revanche préservée et la détection fonctionne.

    On écarte donc les plages listées dans `RESEAUX_NON_ANNONCABLES`
    (par défaut `172.16.0.0/12`, celle où Docker place ses réseaux bridge).
    Mieux vaut ne rien enregistrer qu'une adresse fausse mais crédible : un
    champ vide se voit, `172.18.0.1:9001` passe inaperçu.

    Réglage plutôt que détection dynamique : la règle doit donner le même
    résultat que Django tourne dans un conteneur ou directement sur l'hôte.
    Une PME dont le LAN est réellement en 172.16/12 renseignera `NODE_ADDR`.
    """
    if not _est_une_ip(ip):
        return False

    adresse = ipaddress.ip_address(ip)
    for prefixe in getattr(settings, 'RESEAUX_NON_ANNONCABLES', ['172.16.0.0/12']):
        try:
            if adresse in ipaddress.ip_network(prefixe, strict=False):
                return False
        except ValueError:
            continue
    return True


def _resoudre_adresse(annoncee: str, ip_client: str, port_defaut: str) -> str:
    """Complète l'adresse annoncée par le nœud avec l'IP vue par le serveur.

    Si le nœud a annoncé une vraie IP, on la respecte — il peut avoir une bonne
    raison (machine multi-domiciliée, adresse de publication différente). Sinon
    — champ vide, ou gabarit `<IP_VM>` laissé tel quel dans le compose — on
    substitue l'adresse source de la requête en conservant le port annoncé.
    """
    hote, _, port = (annoncee or '').rpartition(':')
    if not port.isdigit():
        hote, port = (annoncee or ''), port_defaut

    if _est_une_ip(hote):
        return annoncee
    if not _ip_exploitable(ip_client):
        # Ni le nœud ni le serveur ne connaissent l'adresse : on laisse vide
        # plutôt que d'enregistrer un gabarit ou une adresse du réseau Docker.
        return ''
    return f'{ip_client}:{port}'


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
      - node_kind (str, optionnel)   : « serveur » (défaut) ou « temoin » (ADR-005)
      - etat_quorum (str, optionnel) : « sain », « sous_le_seuil » ou « perdu » (étape 2d)
      - etat_application (str, opt.) : « ok » ou « ko » — application hébergée (étape 3)
      - chiffrement_disque (str, opt.) : « oui », « non » ou « inconnu » — disque de la machine (étape 4)
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

    # Créer ou retrouver la machine par installation_id.
    # Les adresses annoncées sont complétées par l'IP source vue du serveur : le
    # nœud, enfermé dans son conteneur, ne connaît pas l'IP LAN de son hôte.
    ip_client = _ip_client(request)
    node_addr = _resoudre_adresse(request.data.get('node_addr', ''), ip_client, '9001')
    web_addr = _resoudre_adresse(request.data.get('web_addr', ''), ip_client, '3000')
    node_role = request.data.get('node_role', '')
    node_kind = request.data.get('node_kind') or 'serveur'
    if node_kind not in ('serveur', 'temoin'):
        return Response(
            {'status': 'error', 'message': 'node_kind invalide (attendu : serveur ou temoin).'},
            status=status.HTTP_400_BAD_REQUEST,
        )
    chiffrement_disque = request.data.get('chiffrement_disque', '')
    if chiffrement_disque not in ('', 'oui', 'non', 'inconnu'):
        return Response(
            {'status': 'error', 'message': 'chiffrement_disque invalide.'},
            status=status.HTTP_400_BAD_REQUEST,
        )
    etat_application = request.data.get('etat_application', '')
    if etat_application not in ('', 'ok', 'ko'):
        return Response(
            {'status': 'error', 'message': 'etat_application invalide.'},
            status=status.HTTP_400_BAD_REQUEST,
        )
    etat_quorum = request.data.get('etat_quorum', '')
    if etat_quorum not in ('', 'sain', 'sous_le_seuil', 'perdu'):
        return Response(
            {'status': 'error', 'message': 'etat_quorum invalide.'},
            status=status.HTTP_400_BAD_REQUEST,
        )
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
            'node_kind': node_kind,
            'etat_quorum': etat_quorum,
            'etat_application': etat_application,
            'chiffrement_disque': chiffrement_disque,
            'is_active': True,
        },
    )

    # Si la machine existait déjà, mettre à jour les champs et last_seen (auto_now)
    if not created:
        device.hostname = request.data.get('hostname', device.hostname)
        device.os = request.data.get('os', device.os)
        device.node_kind = node_kind
        if etat_quorum:
            device.etat_quorum = etat_quorum
        if etat_application:
            device.etat_application = etat_application
        if chiffrement_disque:
            device.chiffrement_disque = chiffrement_disque
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
            # Un seul primaire par tenant : rétrograder les autres nœuds primary → standby.
            # Couvre le split-brain après redémarrage du serveur SaaS.
            if node_role == 'primary':
                Device.objects.filter(
                    tenant=tenant,
                    node_role='primary',
                    is_active=True,
                ).exclude(installation_id=installation_id).update(node_role='standby')
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

    # Vérifier les sièges disponibles (après get_or_create pour inclure la machine actuelle).
    # Un témoin etcd ne consomme pas de poste (ADR-005).
    active_device_count = tenant.devices.filter(is_active=True).exclude(node_kind='temoin').count()
    if node_kind != 'temoin' and active_device_count > active_license.seats:
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
