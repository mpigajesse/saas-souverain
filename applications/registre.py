"""Présence d'une image dans le registre de l'éditeur AMANE.

Une application n'est validée que si son image, épinglée par digest, peut
effectivement être tirée par les machines des PME : on interroge l'API v2 du
registre (`HEAD /v2/<dépôt>/manifests/<digest>`).
"""
import urllib.error
import urllib.request

from django.conf import settings

# Les formats de manifeste qu'un registre peut servir pour un digest.
_ACCEPT = ', '.join([
    'application/vnd.oci.image.index.v1+json',
    'application/vnd.oci.image.manifest.v1+json',
    'application/vnd.docker.distribution.manifest.list.v2+json',
    'application/vnd.docker.distribution.manifest.v2+json',
])


class RegistreInjoignable(RuntimeError):
    """Aucune adresse du registre n'a répondu."""


def _candidats() -> list[str]:
    # Mêmes adresses que `dashboard.api_images` : nom de service compose dans
    # un conteneur, `localhost` en `runserver` sur l'hôte.
    port = getattr(settings, 'REGISTRY_PORT', '5000')
    return [
        getattr(settings, 'REGISTRY_INTERNAL_URL', 'http://registry:5000'),
        f'http://localhost:{port}',
    ]


def image_presente(image: str) -> bool:
    """`dépôt@sha256:…` est-il publié dans le registre ? Lève
    `RegistreInjoignable` si le registre ne répond pas."""
    depot, digest = image.split('@', 1)
    joint = False
    for base in _candidats():
        url = f"{base.rstrip('/')}/v2/{depot}/manifests/{digest}"
        requete = urllib.request.Request(url, method='HEAD', headers={'Accept': _ACCEPT})
        try:
            with urllib.request.urlopen(requete, timeout=3) as reponse:
                return 200 <= reponse.status < 300
        except urllib.error.HTTPError as e:
            joint = True
            if e.code == 404:
                return False
        except (urllib.error.URLError, TimeoutError, OSError):
            continue
    if joint:
        return False
    raise RegistreInjoignable('registre injoignable')
