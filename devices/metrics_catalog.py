"""
Catalogue explicatif des métriques supervisées.

Transforme des métriques brutes (clé → valeur) en lignes lisibles par un
administrateur : libellé, valeur formatée, seuil, état (sain/surveiller/
critique), et signification en langage clair. Utilisé pour détailler les
rapports et l'interface.
"""
from __future__ import annotations

# ── Serveur éditeur (host) ───────────────────────────────────────────────────
# Chaque entrée : (label, unité, signification, seuil_surveiller, seuil_critique)
HOST_CATALOG = {
    'cpu_percent': ("Processeur (CPU)", "%",
                    "Charge globale du processeur du serveur éditeur.", 75, 90),
    'mem_percent': ("Mémoire vive (RAM)", "%",
                    "Taux d'occupation de la mémoire. Au-delà, le système ralentit.", 80, 92),
    'disk_percent': ("Disque", "%",
                     "Espace utilisé sur le volume de l'application.", 85, 95),
    'uptime_h': ("Disponibilité (uptime)", "h",
                 "Durée écoulée depuis le dernier redémarrage du serveur.", None, None),
    'process_count': ("Processus actifs", "",
                      "Nombre de processus en cours d'exécution sur l'hôte.", None, None),
    'db_ok': ("Base de données SaaS", "",
              "Connexion à PostgreSQL de l'éditeur (comptes, licences, parc).", None, None),
    'tenants_actifs': ("Tenants actifs", "",
                       "Nombre d'entreprises clientes actuellement actives.", None, None),
    'devices_actifs': ("Nœuds enregistrés", "",
                       "Total des machines PME actives dans le parc.", None, None),
    'devices_en_ligne': ("Nœuds en ligne", "",
                         "Machines PME ayant émis un battement de cœur récemment.", None, None),
}

# ── Cluster PME (features produites par ai_monitor.build_features) ────────────
CLUSTER_CATALOG = {
    'health_actuel': ("État courant du cluster",
                      "Diagnostic technique instantané (ok, replication_down, no_standby…)."),
    'nombre_noeuds': ("Nombre de nœuds",
                      "Total de machines actives dans le cluster de cette PME."),
    'noeuds_en_ligne': ("Nœuds en ligne",
                        "Nœuds ayant communiqué dans les 2 dernières minutes."),
    'standbys_en_streaming': ("Standbys en réplication",
                              "Réplicas réellement alimentés en WAL par le primaire "
                              "(source de vérité de la réplication). 0 = aucune redondance."),
    'echantillons_collectes_6h': ("Échantillons collectés (6 h)",
                                  "Nombre de mesures accumulées sur 6 h — la matière analysée."),
    'streaming_min_6h': ("Réplication minimale (6 h)",
                         "Plus faible nombre de standbys en streaming observé sur la période."),
    'streaming_max_6h': ("Réplication maximale (6 h)",
                         "Plus grand nombre de standbys en streaming observé sur la période."),
    'ecart_heartbeat_max_s': ("Écart max. entre heartbeats (s)",
                              "Plus long silence entre deux mesures. Au-delà de ~180 s, "
                              "signe d'un nœud instable ou d'un réseau dégradé."),
    'basculements_recents_6h': ("Bascules récentes (6 h)",
                                "Nombre de promotions standby→primaire détectées. "
                                ">0 indique une instabilité de leadership."),
}

STATE_LABELS = {'sain': 'Normal', 'surveiller': 'À surveiller', 'critique': 'Critique', 'info': 'Information'}


def _fmt(value, unit: str) -> str:
    if isinstance(value, bool):
        return "Opérationnelle" if value else "Injoignable"
    if unit:
        return f"{value} {unit}".strip()
    return str(value)


def _state_from_thresholds(value, warn, crit) -> str:
    try:
        v = float(value)
    except (TypeError, ValueError):
        return 'info'
    if crit is not None and v >= crit:
        return 'critique'
    if warn is not None and v >= warn:
        return 'surveiller'
    return 'sain'


def explain_host(metrics: dict) -> list[dict]:
    """Retourne une liste de lignes détaillées pour les métriques hôte."""
    rows = []
    for key, (label, unit, meaning, warn, crit) in HOST_CATALOG.items():
        if key not in metrics:
            continue
        value = metrics[key]
        if key == 'db_ok':
            state = 'sain' if value else 'critique'
            threshold = "Doit rester opérationnelle"
        elif warn is not None:
            state = _state_from_thresholds(value, warn, crit)
            threshold = f"Vigilance ≥ {warn}{unit} · Critique ≥ {crit}{unit}"
        else:
            state = 'info'
            threshold = "—"
        rows.append({
            'label': label,
            'value': _fmt(value, unit),
            'threshold': threshold,
            'state': state,
            'state_label': STATE_LABELS[state],
            'meaning': meaning,
        })
    return rows


def explain_cluster(features: dict) -> list[dict]:
    """Retourne une liste de lignes détaillées pour les features d'un cluster."""
    rows = []
    for key, value in features.items():
        label, meaning = CLUSTER_CATALOG.get(key, (key, "Métrique technique."))
        state = 'info'
        if key == 'standbys_en_streaming':
            state = 'critique' if value in (0, None) else 'sain'
        elif key == 'ecart_heartbeat_max_s':
            try:
                state = 'surveiller' if float(value) > 180 else 'sain'
            except (TypeError, ValueError):
                state = 'info'
        elif key == 'basculements_recents_6h':
            try:
                state = 'surveiller' if int(value) > 0 else 'sain'
            except (TypeError, ValueError):
                state = 'info'
        rows.append({
            'label': label,
            'value': '—' if value is None else str(value),
            'state': state,
            'state_label': STATE_LABELS[state],
            'meaning': meaning,
        })
    return rows
