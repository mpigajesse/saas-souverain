"""Manifeste d'une application externe hébergée par AMANE (refonte V3,
ADR-002, ADR-007).

Le manifeste décrit, sans rien modifier à l'application, ce dont AMANE a besoin
pour l'exécuter :

    {
      "type_artefact": "docker",                       # binaire / exe : plus tard (ADR-002)
      "image": "editeurs/facturation@sha256:<64 hex>", # épinglée par digest
      "commande": ["/app/serveur", "--port", "8080"],  # commande d'origine de l'image
      "port": 8080,                                    # port de son interface web
      "sante": "/sante",                               # chemin sondé par le nœud
      "variables": {"APP_ENV": "production"},
      "base_de_donnees": {                             # facultatif
        "variables": {"DATABASE_URL": "postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}"}
      },
      "volumes": [{"nom": "fichiers", "chemin": "/data"}]
    }

`valider()` refuse tout manifeste incomplet ou dangereux ; `pour_compose()`
le met en forme pour le gabarit de la pile (valeurs YAML sûres, `$` échappé
pour Docker Compose).
"""
import json
import re

TYPES_PRIS_EN_CHARGE = ('docker',)
TYPES_PREVUS = ('binaire', 'exe')  # ADR-002 : formats acceptés plus tard

_IMAGE = re.compile(r'^[a-z0-9][a-z0-9._/-]*@sha256:[0-9a-f]{64}$')
_VARIABLE = re.compile(r'^[A-Z_][A-Z0-9_]{0,63}$')
_VOLUME = re.compile(r'^[a-z0-9][a-z0-9-]{0,39}$')
_CHEMIN = re.compile(r'^/[A-Za-z0-9._/-]*$')
_ESPACES_RESERVES = {'utilisateur', 'mot_de_passe', 'hote', 'port', 'base'}
_ESPACE_RESERVE = re.compile(r'\{([a-z_]+)\}')


class ManifesteInvalide(ValueError):
    """Le manifeste ne peut pas être hébergé tel quel."""


def _variables(valeur, chemin: str) -> dict:
    if valeur is None:
        return {}
    if not isinstance(valeur, dict):
        raise ManifesteInvalide(f'{chemin} doit être un objet')
    for nom, contenu in valeur.items():
        if not _VARIABLE.match(nom) or nom.startswith('AMANE_'):
            raise ManifesteInvalide(f'{chemin} : nom de variable refusé « {nom} » (MAJUSCULES, AMANE_ réservé)')
        if not isinstance(contenu, str) or '\n' in contenu:
            raise ManifesteInvalide(f'{chemin}.{nom} doit être une chaîne sur une ligne')
    return dict(valeur)


def valider(manifeste) -> dict:
    """Vérifie le manifeste et renvoie sa forme normalisée."""
    if not isinstance(manifeste, dict):
        raise ManifesteInvalide('le manifeste doit être un objet JSON')
    type_artefact = manifeste.get('type_artefact', 'docker')
    if type_artefact in TYPES_PREVUS:
        raise ManifesteInvalide(f'type « {type_artefact} » pas encore pris en charge (ADR-002 : images Docker d\'abord)')
    if type_artefact not in TYPES_PRIS_EN_CHARGE:
        raise ManifesteInvalide(f'type d\'artefact inconnu : {type_artefact!r}')

    image = manifeste.get('image', '')
    if not isinstance(image, str) or not _IMAGE.match(image):
        raise ManifesteInvalide('image : référence épinglée par digest attendue (dépôt@sha256:…)')

    commande = manifeste.get('commande')
    if (not isinstance(commande, list) or not commande
            or not all(isinstance(c, str) and c and '\n' not in c for c in commande)):
        raise ManifesteInvalide('commande : liste non vide de chaînes (commande d\'origine de l\'image)')

    port = manifeste.get('port')
    if not isinstance(port, int) or isinstance(port, bool) or not 1 <= port <= 65535:
        raise ManifesteInvalide('port : entier entre 1 et 65535')

    sante = manifeste.get('sante', '/')
    if not isinstance(sante, str) or not _CHEMIN.match(sante):
        raise ManifesteInvalide('sante : chemin absolu (ex. /sante)')

    variables = _variables(manifeste.get('variables'), 'variables')

    base = manifeste.get('base_de_donnees')
    modeles = None
    if base is not None:
        if not isinstance(base, dict):
            raise ManifesteInvalide('base_de_donnees doit être un objet')
        modeles = _variables(base.get('variables'), 'base_de_donnees.variables')
        if not modeles:
            raise ManifesteInvalide('base_de_donnees.variables : au moins une variable (ex. DATABASE_URL)')
        for nom, modele in modeles.items():
            inconnus = set(_ESPACE_RESERVE.findall(modele)) - _ESPACES_RESERVES
            if inconnus:
                raise ManifesteInvalide(f'base_de_donnees.variables.{nom} : espace réservé inconnu {sorted(inconnus)}')
            if nom in variables:
                raise ManifesteInvalide(f'variable {nom} déclarée deux fois')

    volumes = manifeste.get('volumes') or []
    if not isinstance(volumes, list):
        raise ManifesteInvalide('volumes doit être une liste')
    noms = set()
    for v in volumes:
        if (not isinstance(v, dict) or not _VOLUME.match(str(v.get('nom', '')))
                or not _CHEMIN.match(str(v.get('chemin', ''))) or v.get('chemin') == '/'):
            raise ManifesteInvalide('volumes : {"nom": minuscules-et-tirets, "chemin": chemin absolu}')
        if v['nom'] in noms:
            raise ManifesteInvalide(f'volume « {v["nom"]} » déclaré deux fois')
        noms.add(v['nom'])

    return {
        'type_artefact': type_artefact,
        'image': image,
        'commande': list(commande),
        'port': port,
        'sante': sante,
        'variables': variables,
        'base_de_donnees': {'variables': modeles} if modeles is not None else None,
        'volumes': [{'nom': v['nom'], 'chemin': v['chemin']} for v in volumes],
    }


def _yaml(valeur: str) -> str:
    """Chaîne YAML sûre (JSON est un sous-ensemble de YAML) ; `$` doublé pour
    que Docker Compose ne l'interprète pas."""
    return json.dumps(valeur.replace('$', '$$'), ensure_ascii=False)


def pour_compose(manifeste: dict, registre: str) -> dict:
    """Contexte du gabarit de la pile pour un manifeste validé."""
    m = valider(manifeste)
    environnement = [(nom, _yaml(valeur)) for nom, valeur in sorted(m['variables'].items())]
    base = m['base_de_donnees']
    if base:
        environnement.append(('AMANE_MODELES', _yaml(json.dumps(base['variables'], ensure_ascii=False, sort_keys=True))))
    return {
        'image': f'{registre}/{m["image"]}',
        'commande': _yaml_liste(m['commande']),
        'port': m['port'],
        'sante': m['sante'],
        'environnement': environnement,
        'base': bool(base),
        'volumes': [{'cle': f'app-{v["nom"]}', 'chemin': v['chemin']} for v in m['volumes']],
    }


def _yaml_liste(elements) -> str:
    return '[' + ', '.join(_yaml(e) for e in elements) + ']'
