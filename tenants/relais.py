"""Jeton du relais propre à chaque tenant (refonte V3, étape 9, ADR-011).

Le portail et le relais partagent un secret (`RELAY_SECRET`). Le jeton d'un
tenant est `HMAC-SHA256(secret, "amane-relais/v1/" + tenant)` en hexadécimal :
le portail le remet à la pile du tenant, le relais le recalcule à chaque
requête (`ss_crypto::jeton_relais`, même vecteur de test des deux côtés). Il
ouvre les blobs et la topologie de ce seul tenant.

Le jeton ne donne accès à aucune donnée en clair : les blobs sont chiffrés par
la DEK du cluster, que ni le portail ni le relais ne détiennent.
"""
import hashlib
import hmac
import logging

from django.conf import settings

logger = logging.getLogger(__name__)

# Séparation de domaine, identique à `spike/crates/ss-crypto/src/jeton_relais.rs`.
CONTEXTE = b'amane-relais/v1/'


_absence_signalee = False


def jeton_relais(tenant_id) -> str:
    """Jeton du tenant, ou chaîne vide si `RELAY_SECRET` n'est pas défini
    (relais de laboratoire en mode ouvert)."""
    global _absence_signalee
    secret = getattr(settings, 'RELAY_SECRET', '').strip()
    if not secret:
        if not _absence_signalee:
            _absence_signalee = True
            logger.warning(
                "RELAY_SECRET absent : les piles n'auront pas de jeton du relais ; "
                "un relais qui exige l'authentification refusera leurs sauvegardes",
            )
        return ''
    return hmac.new(secret.encode('utf-8'), CONTEXTE + str(tenant_id).encode('ascii'), hashlib.sha256).hexdigest()


def entetes_relais(tenant_id) -> dict:
    """En-têtes HTTP d'une requête du portail au relais pour ce tenant."""
    jeton = jeton_relais(tenant_id)
    return {'x-relay-token': jeton} if jeton else {}
