"""Voie PME — `/api/tenants/pme/...`, authentifiée par jeton d'enregistrement.

Les installeurs tournent sur une machine sans compte éditeur : pas de session,
seulement le jeton d'enregistrement du tenant.
"""
import uuid

from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response

from .api_espace import reponse_pile_non_publiee, type_machine_demande
from .models import Tenant
from .pile import PileNonPubliee
from .services import build_compose_response


@api_view(['GET'])
@permission_classes([AllowAny])
def compose_telecharger_pme(request):
    """`GET /api/tenants/pme/compose/?token=<registration_token>[&type=temoin]`.

    Voie de repli des installeurs PME, calquée sur
    `dashboard.api_images.node_image_telecharger_pme` : la machine qui déploie
    le nœud n'a aucune session éditeur, elle ne peut donc pas passer par
    `/api/tenants/me/download/compose/`. Elle s'authentifie avec le seul secret
    dont elle dispose — le jeton d'enregistrement du tenant — exactement comme
    `devices/api_cluster_status.py`.

    C'est ce point d'entrée qui fait de `templates/tenants/docker-compose.yml.tmpl`
    la définition UNIQUE de la pile : les deux installeurs la téléchargent au
    lieu de la régénérer chacun à sa façon, donc Windows et Linux déploient
    rigoureusement les mêmes services.
    """
    token_raw = request.GET.get('token', '').strip()
    if not token_raw:
        return Response({'detail': 'Paramètre token manquant.'}, status=400)

    try:
        uuid.UUID(token_raw)
    except ValueError:
        return Response({'detail': 'Format UUID invalide.'}, status=400)

    try:
        tenant = Tenant.objects.get(registration_token=token_raw, is_active=True)
    except Tenant.DoesNotExist:
        return Response({'detail': 'Token invalide ou tenant inactif.'}, status=403)

    type_machine = type_machine_demande(request)
    if type_machine is None:
        return Response({'detail': "type doit être 'serveur' ou 'temoin'."}, status=400)
    try:
        return build_compose_response(request, tenant, type_machine)
    except PileNonPubliee as exc:
        return reponse_pile_non_publiee(exc)
