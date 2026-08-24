"""
Agent de supervision intelligente du cluster PME.

Analyse les métriques techniques d'un cluster (rôles PostgreSQL, réplication
streaming, failover, régularité des heartbeats) et produit un diagnostic en
langage naturel + un score de risque, via l'API Mistral (free tier).

Garanties :
  - Zero-knowledge : seules des métriques d'INFRASTRUCTURE sont transmises au
    modèle. Aucune donnée métier (stock, factures, clients) ne sort jamais.
  - Fail-safe : si la clé API est absente, le réseau échoue ou le quota est
    épuisé, un repli local déterministe prend le relais. Le dashboard ne casse
    jamais.
"""
from __future__ import annotations

import json
import logging
from dataclasses import dataclass, asdict
from datetime import timedelta

import certifi
import requests
from django.conf import settings
from django.utils import timezone

logger = logging.getLogger(__name__)

MISTRAL_ENDPOINT = "https://api.mistral.ai/v1/chat/completions"
MISTRAL_MODEL = "mistral-small-latest"  # disponible en free tier
REQUEST_TIMEOUT = 12  # secondes — on échoue vite vers le repli local
HISTORY_WINDOW = timedelta(hours=6)
MAX_SAMPLES = 60  # garde le prompt compact (free tier)

RISK_LEVELS = ("sain", "surveiller", "critique")


@dataclass(frozen=True)
class MonitorVerdict:
    """Résultat d'une analyse de supervision."""
    risk_level: str          # "sain" | "surveiller" | "critique"
    anomaly_score: int       # 0–100
    summary: str             # diagnostic court
    recommendation: str      # action préventive recommandée
    source: str              # "mistral" | "local"
    details: str = ""        # raisonnement détaillé de l'agent (pour le rapport)

    def as_dict(self) -> dict:
        return asdict(self)


# ─────────────────────────────────────────────────────────────────────────────
# Extraction des caractéristiques (features) à partir de l'historique
# ─────────────────────────────────────────────────────────────────────────────

def build_features(tenant, snapshot: dict) -> dict:
    """
    Construit un dictionnaire de métriques techniques à partir de l'état courant
    et de la série temporelle récente. Aucune donnée métier.
    """
    from .models import ClusterMetricSample

    since = timezone.now() - HISTORY_WINDOW
    samples = list(
        ClusterMetricSample.objects
        .filter(tenant=tenant, captured_at__gte=since)
        .order_by('captured_at')[:MAX_SAMPLES]
    )

    streaming_series = [s.streaming_standby_count for s in samples]
    failover_series = [s.failover_count for s in samples]

    # Régularité des heartbeats : écart max entre deux échantillons consécutifs.
    gaps = []
    for prev, cur in zip(samples, samples[1:]):
        gaps.append((cur.captured_at - prev.captured_at).total_seconds())
    max_gap = max(gaps) if gaps else 0.0

    # Le compteur de failover ne fait que croître : un saut = bascule récente.
    failover_jumps = sum(
        1 for a, b in zip(failover_series, failover_series[1:]) if b > a
    )

    return {
        "health_actuel": snapshot.get("health"),
        "nombre_noeuds": snapshot.get("total", 0),
        "noeuds_en_ligne": snapshot.get("online_count", 0),
        "standbys_en_streaming": snapshot.get("streaming", 0),
        "echantillons_collectes_6h": len(samples),
        "streaming_min_6h": min(streaming_series) if streaming_series else None,
        "streaming_max_6h": max(streaming_series) if streaming_series else None,
        "ecart_heartbeat_max_s": round(max_gap, 1),
        "basculements_recents_6h": failover_jumps,
    }


# ─────────────────────────────────────────────────────────────────────────────
# Appel Mistral
# ─────────────────────────────────────────────────────────────────────────────

_SYSTEM_PROMPT = (
    "Tu es un agent de supervision SRE spécialisé dans les clusters PostgreSQL "
    "répliqués (primaire/standby). On te fournit UNIQUEMENT des métriques "
    "d'infrastructure — jamais de données applicatives. Analyse l'état et la "
    "dynamique récente, puis réponds STRICTEMENT en JSON avec les clés : "
    '{"risk_level": "sain|surveiller|critique", "anomaly_score": 0-100, '
    '"summary": "diagnostic en une à deux phrases", '
    '"recommendation": "action préventive concrète en une phrase", '
    '"details": "explication pédagogique du raisonnement en 3 à 5 phrases : '
    "quelles métriques ont été examinées, ce qu'elles indiquent, et pourquoi ce "
    'niveau de risque a été retenu"}. '
    "Pas de texte hors JSON. Réponds en français."
)


def _chat_json(system_prompt: str, user_content: str) -> dict:
    """Appel Mistral générique renvoyant un objet JSON parsé."""
    payload = {
        "model": MISTRAL_MODEL,
        "temperature": 0.2,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_content},
        ],
    }
    headers = {
        "Authorization": f"Bearer {settings.MISTRAL_API_KEY}",
        "Content-Type": "application/json",
    }
    # verify explicite : ignore une éventuelle variable d'environnement
    # (REQUESTS_CA_BUNDLE/SSL_CERT_FILE) pointant vers un bundle CA invalide.
    resp = requests.post(
        MISTRAL_ENDPOINT, json=payload, headers=headers,
        timeout=REQUEST_TIMEOUT, verify=certifi.where(),
    )
    resp.raise_for_status()
    content = resp.json()["choices"][0]["message"]["content"]
    return json.loads(content)


def _verdict_from_data(data: dict) -> MonitorVerdict:
    """Construit un MonitorVerdict robuste à partir d'un JSON de modèle."""
    risk = str(data.get("risk_level", "")).strip().lower()
    if risk not in RISK_LEVELS:
        risk = "surveiller"
    try:
        score = int(data.get("anomaly_score", 0))
    except (TypeError, ValueError):
        score = 0
    score = max(0, min(100, score))
    return MonitorVerdict(
        risk_level=risk,
        anomaly_score=score,
        summary=str(data.get("summary", "")).strip() or "Analyse indisponible.",
        recommendation=str(data.get("recommendation", "")).strip() or "—",
        source="mistral",
        details=str(data.get("details", "")).strip(),
    )


def _call_mistral(features: dict, tenant_name: str) -> MonitorVerdict:
    data = _chat_json(
        _SYSTEM_PROMPT,
        f"Cluster du tenant « {tenant_name} ». "
        f"Métriques techniques :\n{json.dumps(features, ensure_ascii=False, indent=2)}",
    )
    return _verdict_from_data(data)


# ─────────────────────────────────────────────────────────────────────────────
# Repli local déterministe (jamais d'échec visible côté dashboard)
# ─────────────────────────────────────────────────────────────────────────────

def _local_fallback(features: dict) -> MonitorVerdict:
    health = features.get("health_actuel")
    streaming = features.get("standbys_en_streaming", 0) or 0
    max_gap = features.get("ecart_heartbeat_max_s", 0) or 0
    failovers = features.get("basculements_recents_6h", 0) or 0

    if health in ("replication_down", "no_primary", "primary_offline"):
        return MonitorVerdict(
            "critique", 90,
            "Anomalie critique détectée sur l'état de réplication ou la disponibilité du primaire.",
            "Vérifier le flux WAL et la disponibilité des nœuds ; intervenir sans délai.",
            "local",
            details=(
                f"L'état mesuré du cluster est « {health} ». Le nombre de standbys en "
                f"streaming est {streaming}, ce qui signifie qu'aucune redondance fiable "
                "n'est garantie. Sans réplication active ou sans primaire disponible, une "
                "panne entraînerait une perte de données — d'où le niveau critique."
            ),
        )
    if failovers > 0 or max_gap > 180:
        return MonitorVerdict(
            "surveiller", 55,
            "Signaux de fragilité : bascule récente ou heartbeats irréguliers.",
            "Surveiller le standby et la latence réseau ; préparer une vérification du fencing.",
            "local",
            details=(
                f"Sur les 6 dernières heures, {failovers} bascule(s) ont été observée(s) et "
                f"l'écart maximal entre deux heartbeats a atteint {max_gap:.0f} s (seuil de "
                "vigilance : 180 s). Ces signaux précèdent souvent un incident : le cluster "
                "fonctionne mais montre une instabilité à surveiller."
            ),
        )
    if health == 'no_standby' or streaming == 0:
        return MonitorVerdict(
            "surveiller", 40,
            "Aucune redondance active : pas de standby en streaming.",
            "Ajouter ou reconnecter un standby pour restaurer la résilience.",
            "local",
            details=(
                f"Le primaire fonctionne mais le nombre de standbys en streaming est {streaming}. "
                "Le cluster accepte les écritures, cependant aucune copie n'est tenue à jour en "
                "temps réel : une panne du primaire serait non récupérable automatiquement."
            ),
        )
    return MonitorVerdict(
        "sain", 8,
        "Cluster stable : réplication active et heartbeats réguliers.",
        "Aucune action requise ; poursuivre la collecte de métriques.",
        "local",
        details=(
            f"Le cluster est en état « {health} », avec {streaming} standby(s) en streaming et "
            f"un écart de heartbeat maximal de {max_gap:.0f} s, sous le seuil de vigilance. "
            "Aucune bascule récente n'a été détectée : la réplication et la résilience sont nominales."
        ),
    )


# ─────────────────────────────────────────────────────────────────────────────
# Point d'entrée public
# ─────────────────────────────────────────────────────────────────────────────

def analyze_with_features(tenant, snapshot: dict) -> tuple[MonitorVerdict, dict]:
    """
    Analyse un cluster et renvoie (verdict, features). Les features sont les
    métriques techniques effectivement transmises au modèle — utiles pour
    l'affichage et le rapport. Bascule sur le repli local si Mistral échoue.
    """
    features = build_features(tenant, snapshot)

    if not getattr(settings, "MISTRAL_API_KEY", ""):
        logger.info("MISTRAL_API_KEY absente — repli local pour le tenant %s", tenant.pk)
        return _local_fallback(features), features

    try:
        return _call_mistral(features, tenant.name), features
    except Exception as exc:  # réseau, quota, JSON invalide…
        logger.warning("Appel Mistral échoué (%s) — repli local", exc)
        return _local_fallback(features), features


def analyze_cluster(tenant, snapshot: dict) -> MonitorVerdict:
    """Analyse un cluster et renvoie uniquement le verdict."""
    verdict, _features = analyze_with_features(tenant, snapshot)
    return verdict


# ─────────────────────────────────────────────────────────────────────────────
# Supervision du serveur éditeur SaaS (la machine hôte)
# ─────────────────────────────────────────────────────────────────────────────

_HOST_SYSTEM_PROMPT = (
    "Tu es un agent SRE supervisant le SERVEUR ÉDITEUR d'une plateforme SaaS "
    "(machine hôte Django + PostgreSQL). On te fournit des métriques système et "
    "applicatives. Évalue la santé d'exploitation et réponds STRICTEMENT en JSON : "
    '{"risk_level": "sain|surveiller|critique", "anomaly_score": 0-100, '
    '"summary": "diagnostic en une à deux phrases", '
    '"recommendation": "action concrète en une phrase", '
    '"details": "explication pédagogique en 3 à 5 phrases : quelles métriques '
    "(CPU, RAM, disque, base, parc) ont été examinées, ce qu'elles révèlent, et "
    'pourquoi ce niveau de risque"}. '
    "Pas de texte hors JSON. Réponds en français."
)


def _host_fallback(metrics: dict) -> MonitorVerdict:
    from .host_monitor import host_health

    health = host_health(metrics)
    cpu = metrics.get('cpu_percent', 0)
    mem = metrics.get('mem_percent', 0)
    disk = metrics.get('disk_percent', 0)
    db = "opérationnelle" if metrics.get('db_ok', True) else "INJOIGNABLE"
    base_detail = (
        f"CPU à {cpu} %, mémoire à {mem} %, disque à {disk} %, base de données {db}."
    )
    if health == 'critique':
        score = 88 if metrics.get('db_ok', True) else 97
        return MonitorVerdict(
            "critique", score,
            "Serveur éditeur sous forte tension (ressources saturées ou base injoignable).",
            "Libérer des ressources (CPU/RAM/disque) ou rétablir la connexion à la base sans délai.",
            "local",
            details=(
                f"{base_detail} Au moins une ressource dépasse le seuil critique (CPU ≥ 90 %, "
                "RAM ≥ 92 %, disque ≥ 95 %) ou la base est injoignable. Le service éditeur "
                "(comptes, licences, supervision) risque une interruption."
            ),
        )
    if health == 'surveiller':
        return MonitorVerdict(
            "surveiller", 45,
            "Charge élevée sur le serveur éditeur : surveiller l'évolution.",
            "Surveiller CPU/RAM/disque ; anticiper un redimensionnement si la tendance persiste.",
            "local",
            details=(
                f"{base_detail} Une ressource franchit le seuil de vigilance (CPU ≥ 75 %, "
                "RAM ≥ 80 %, disque ≥ 85 %). Le serveur reste fonctionnel mais la marge se "
                "réduit : à surveiller pour éviter une saturation."
            ),
        )
    return MonitorVerdict(
        "sain", 6,
        "Serveur éditeur stable : ressources confortables et base opérationnelle.",
        "Aucune action requise.",
        "local",
        details=(
            f"{base_detail} Toutes les ressources sont sous les seuils de vigilance et la "
            "base répond. Le serveur éditeur dispose d'une marge confortable."
        ),
    )


def analyze_host(metrics: dict) -> MonitorVerdict:
    """Analyse l'état du serveur éditeur. Repli local si Mistral indisponible."""
    if not getattr(settings, "MISTRAL_API_KEY", ""):
        return _host_fallback(metrics)
    try:
        data = _chat_json(
            _HOST_SYSTEM_PROMPT,
            "Serveur éditeur SaaS. Métriques :\n"
            f"{json.dumps(metrics, ensure_ascii=False, indent=2)}",
        )
        return _verdict_from_data(data)
    except Exception as exc:
        logger.warning("Analyse hôte Mistral échouée (%s) — repli local", exc)
        return _host_fallback(metrics)


# ─────────────────────────────────────────────────────────────────────────────
# Supervision du serveur relais zero-knowledge
# ─────────────────────────────────────────────────────────────────────────────

_RELAY_SYSTEM_PROMPT = (
    "Tu es un agent SRE supervisant un SERVEUR RELAIS ZERO-KNOWLEDGE qui stocke "
    "des sauvegardes CHIFFRÉES opaques pour des PME. IMPORTANT : tu ne vois et "
    "n'évalues QUE des métadonnées de santé (disponibilité, uptime, nombre de "
    "tenants ayant des blobs, réseaux) — jamais le contenu, qui est chiffré et "
    "inaccessible par conception. Évalue uniquement si le service est sain et "
    "disponible. Réponds STRICTEMENT en JSON : "
    '{"risk_level": "sain|surveiller|critique", "anomaly_score": 0-100, '
    '"summary": "diagnostic en une à deux phrases", '
    '"recommendation": "action concrète en une phrase", '
    '"details": "explication en 3 à 5 phrases : disponibilité, uptime, stockage, '
    'et confirmation que la souveraineté (zero-knowledge) est préservée"}. '
    "Pas de texte hors JSON. Réponds en français."
)


def _relay_fallback(metrics: dict) -> MonitorVerdict:
    from .relay_monitor import relay_health

    health = relay_health(metrics)
    if not metrics.get('reachable', False):
        return MonitorVerdict(
            "critique", 95,
            "Le serveur relais est injoignable : les sauvegardes chiffrées ne sont plus accessibles.",
            "Vérifier le service relais et la connectivité réseau sans délai.",
            "local",
            details=(
                f"Le relais ({metrics.get('relay_url', '?')}) ne répond pas à son endpoint "
                "de santé. Aucune donnée métier n'est exposée (les blobs restent chiffrés), "
                "mais la disponibilité de la récupération de sinistre n'est plus garantie."
            ),
        )
    if not metrics.get('zero_knowledge', True):
        return MonitorVerdict(
            "critique", 100,
            "Anomalie de sécurité : le relais ne signale plus son mode zero-knowledge.",
            "Suspendre le relais et auditer sa configuration immédiatement.",
            "local",
            details="Le drapeau zero_knowledge est faux — incohérent avec la promesse de souveraineté.",
        )
    return MonitorVerdict(
        "sain", 5,
        "Relais zero-knowledge opérationnel : service disponible, stockage chiffré intact.",
        "Aucune action requise.",
        "local",
        details=(
            f"Le relais répond (uptime {metrics.get('uptime_h', 0)} h, version "
            f"{metrics.get('version', '?')}) et conserve les blobs de {metrics.get('blob_tenants', 0)} "
            "tenant(s). Seules des métadonnées sont lues : le contenu reste chiffré et "
            "inaccessible — souveraineté totale préservée."
        ),
    )


def analyze_relay(metrics: dict) -> MonitorVerdict:
    """Analyse l'état du relais zero-knowledge. Repli local si Mistral indisponible."""
    if not getattr(settings, "MISTRAL_API_KEY", ""):
        return _relay_fallback(metrics)
    try:
        data = _chat_json(
            _RELAY_SYSTEM_PROMPT,
            "Serveur relais zero-knowledge. Métadonnées de santé (aucun contenu de blob) :\n"
            f"{json.dumps(metrics, ensure_ascii=False, indent=2)}",
        )
        return _verdict_from_data(data)
    except Exception as exc:
        logger.warning("Analyse relais Mistral échouée (%s) — repli local", exc)
        return _relay_fallback(metrics)
