"""`GET /api/node-image/` et `GET /api/node-image/download/` — voir CONTRAT-API.md
section 2.

Réservées aux comptes éditeur (staff). L'image du nœud PME est publiée par un
build hors-ligne (`spike/build-node.ps1`) qui dépose `manifeste.json` et
l'archive `.tar.gz` sous `media/node-images/`. Manifeste ou archive absents
sont des états normaux (`disponible: false`), jamais une 500 — même logique
que le relais injoignable dans `dashboard/api_relay.py`. Un registre Docker
local (port 5000, service `registry` dans docker-compose.yml) distribue
ensuite l'image aux machines PME du réseau ; l'interroger reste optionnel :
`registre.joignable: false` est lui aussi un état normal.
"""
import json
import urllib.error
import urllib.request
import uuid
from pathlib import Path

from django.conf import settings
from django.http import FileResponse
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import AllowAny
from rest_framework.response import Response

from api.permissions import IsStaffUser
from tenants.models import Tenant
from tenants.views_web import _saas_host_port

NODE_IMAGES_DIR = 'node-images'
MANIFEST_FILENAME = 'manifeste.json'


def _node_images_dir() -> Path:
    return Path(settings.MEDIA_ROOT) / NODE_IMAGES_DIR


def _load_manifest() -> dict | None:
    """Lit `manifeste.json` s'il existe. Renvoie `None` s'il manque ou s'il
    est illisible — jamais d'exception qui remonterait à l'appelant."""
    manifest_path = _node_images_dir() / MANIFEST_FILENAME
    if not manifest_path.exists():
        return None
    try:
        return json.loads(manifest_path.read_text(encoding='utf-8'))
    except (OSError, json.JSONDecodeError):
        return None


def _archive_exists(manifest: dict) -> bool:
    fichier = manifest.get('fichier')
    if not fichier:
        return False
    return (_node_images_dir() / fichier).exists()


def _query_registry_tags(image: str | None) -> tuple[bool, list]:
    """Interroge l'API du registre Docker pour lister les tags publiés.

    Utilise `REGISTRY_INTERNAL_URL` (nom de service compose) et **non** l'hôte
    public affiché à l'utilisateur : Django tourne dans un conteneur, d'où l'IP
    LAN de la machine hôte n'est pas routable. Sonder cette IP échouerait
    systématiquement et la console annoncerait « registre injoignable » alors
    que les machines du réseau, elles, y accèdent très bien.

    Timeout court : un registre arrêté ne doit jamais figer la page.
    """
    if not image:
        return False, []
    name = image.split(':', 1)[0]

    # Deux contextes d'exécution possibles, et ils n'ont pas la même vue du
    # réseau : dans un conteneur, le registre est joignable par son nom de
    # service compose ; en `runserver` sur la machine hôte, c'est `localhost`
    # qui répond et le nom de service ne résout pas. On essaie l'un puis
    # l'autre plutôt que d'imposer un réglage différent selon le mode.
    port = getattr(settings, 'REGISTRY_PORT', '5000')
    candidats = [
        getattr(settings, 'REGISTRY_INTERNAL_URL', 'http://registry:5000'),
        f'http://localhost:{port}',
    ]

    for base in candidats:
        url = f"{base.rstrip('/')}/v2/{name}/tags/list"
        try:
            with urllib.request.urlopen(url, timeout=2) as resp:
                data = json.loads(resp.read())
            return True, data.get('tags') or []
        except (urllib.error.URLError, TimeoutError, OSError, ValueError):
            continue

    return False, []


@api_view(['GET'])
@permission_classes([IsStaffUser])
def node_image_status(request):
    manifest = _load_manifest()

    saas_host, _ = _saas_host_port(request)
    registry_port = getattr(settings, 'REGISTRY_PORT', '5000')
    registry_host = f"{saas_host}:{registry_port}"

    image = manifest.get('image') if manifest else None
    disponible = bool(manifest) and _archive_exists(manifest)
    joignable, tags = _query_registry_tags(image)

    return Response({
        'disponible': disponible,
        'version': manifest.get('version') if manifest else None,
        'image': image,
        'fichier': manifest.get('fichier') if manifest else None,
        'taille_octets': manifest.get('taille_octets') if manifest else None,
        'sha256': manifest.get('sha256') if manifest else None,
        'construit_le': manifest.get('construit_le') if manifest else None,
        'url_telechargement': '/api/node-image/download/',
        'registre': {
            'hote': registry_host,
            'commande_pull': f"docker pull {registry_host}/{image}" if image else None,
            'tags': tags,
            'joignable': joignable,
        },
    })


def _reponse_archive(manifest: dict | None):
    """Sert l'archive en flux, ou une 404 explicite si elle manque.

    `FileResponse` streame le fichier : l'archive pèse des dizaines de Mo et ne
    doit jamais être chargée en mémoire.
    """
    if manifest is None:
        return Response({'detail': "Aucune image publiée pour le moment."}, status=404)

    fichier = manifest.get('fichier')
    archive_path = _node_images_dir() / fichier if fichier else None
    if not archive_path or not archive_path.exists():
        return Response({'detail': "Archive de l'image introuvable."}, status=404)

    return FileResponse(
        open(archive_path, 'rb'),
        as_attachment=True,
        filename=fichier,
        content_type='application/gzip',
    )


@api_view(['GET'])
@permission_classes([IsStaffUser])
def node_image_download(request):
    return _reponse_archive(_load_manifest())


@api_view(['GET'])
@permission_classes([AllowAny])
def node_image_telecharger_pme(request):
    """`GET /api/node-image/telecharger/?token=<registration_token>`.

    Voie de repli des installeurs PME : la machine qui déploie le nœud n'a
    aucune session éditeur, elle ne peut donc pas passer par
    `/api/node-image/download/`. Elle s'authentifie comme le fait déjà
    `devices/api_cluster_status.py` — par le jeton d'enregistrement du tenant,
    le seul secret dont dispose l'installeur.

    Sert aussi de solution de rechange quand le registre Docker est fermé ou
    qu'un poste Windows ne peut pas le déclarer en `insecure-registries` :
    l'installeur télécharge l'archive et fait `docker load`.
    """
    token_raw = request.GET.get('token', '').strip()
    if not token_raw:
        return Response({'detail': 'Paramètre token manquant.'}, status=400)

    try:
        uuid.UUID(token_raw)
    except ValueError:
        return Response({'detail': 'Format UUID invalide.'}, status=400)

    if not Tenant.objects.filter(registration_token=token_raw, is_active=True).exists():
        return Response({'detail': 'Token invalide ou tenant inactif.'}, status=403)

    return _reponse_archive(_load_manifest())
