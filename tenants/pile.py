"""Images de la pile PME, épinglées par digest (ADR-005, décision 3).

La PME exécute exactement ce que l'éditeur a validé : chaque image du compose
généré est référencée par `<registre>/<dépôt>@sha256:<digest>`, jamais par un
tag mobile. Les digests sont publiés par `scripts/publier-pile.sh`, qui pousse
les images dans le registre de l'éditeur et écrit `media/node-images/pile.json` :

    {"publie_le": "...", "images": {"noeud": "amane/ss-node@sha256:…", ...}}

Sans manifeste complet, aucune pile n'est générée : on refuse plutôt que de
retomber sur des tags non vérifiés.
"""
import json
import re
from pathlib import Path

from django.conf import settings

FICHIER_PILE = 'pile.json'

# Images attendues : nom logique → rôle dans la pile.
IMAGES_PILE = {
    'noeud': 'nœud AMANE (ss-node)',
    'orchestrateur': 'orchestrateur Go (amorçage, superviseur, HAProxy, secrets)',
    'postgresql': 'PostgreSQL 17 + Patroni (Spilo)',
    'etcd': 'etcd (consensus Raft)',
    'haproxy': 'HAProxy (adresse fixe vers le primaire)',
    'pgadmin': 'pgAdmin (consultation locale)',
}

_REFERENCE = re.compile(r'^[a-z0-9][a-z0-9._/-]*@sha256:[0-9a-f]{64}$')


class PileNonPubliee(Exception):
    """Manifeste absent, illisible ou incomplet."""


def _chemin_manifeste() -> Path:
    return Path(settings.MEDIA_ROOT) / 'node-images' / FICHIER_PILE


def images_pile(registre: str) -> dict:
    """Références complètes des images, préfixées par l'hôte du registre de
    l'éditeur tel que la PME le joint. Lève PileNonPubliee si une image manque
    ou n'est pas épinglée par digest."""
    chemin = _chemin_manifeste()
    try:
        manifeste = json.loads(chemin.read_text(encoding='utf-8'))
    except (OSError, json.JSONDecodeError) as exc:
        raise PileNonPubliee(f'manifeste {chemin} absent ou illisible') from exc
    images = manifeste.get('images') or {}
    refs = {}
    for nom in IMAGES_PILE:
        ref = images.get(nom, '')
        if not _REFERENCE.match(ref):
            raise PileNonPubliee(f'image « {nom} » absente ou non épinglée par digest : {ref!r}')
        refs[nom] = f'{registre}/{ref}'
    return refs
