# Branche `feat/agent-supervision-ia` — Agent de supervision IA et livrables de soutenance

> Branche de travail de **MPIGA-ODOUMBA Jesse** · 45 commits
>
> Deux chantiers distincts : un **agent de supervision IA** côté SaaS éditeur, et l'ensemble
> des **livrables de soutenance** (rapport, slides, affiche, scripts de démonstration).

---

## En une phrase

L'éditeur doit pouvoir dire à une PME « ton cluster va tomber » **sans jamais voir ses
données**. Cette branche ajoute un agent qui diagnostique la santé des trois acteurs à partir
de métriques purement techniques, et rend un verdict en langage clair.

---

## Le principe : superviser sans lire

C'est la contrainte structurante du projet. Un agent de supervision classique lit tout ; celui-ci
n'a accès qu'à des compteurs d'infrastructure.

| Acteur supervisé | Ce que l'agent lit | Ce qu'il ne voit jamais |
|---|---|---|
| **Serveur éditeur** | CPU, RAM, disque, uptime, processus, état de la base SaaS | — |
| **Relais zero-knowledge** | `/health` : disponibilité, version, uptime, **nombre** de tenants ayant des blobs | Le contenu des blobs chiffrés |
| **Clusters PME** | Rôle des nœuds, standbys en streaming, écarts de heartbeat, historique des bascules | Stock, factures, clients — rien du métier |

Les deux nouveaux modèles portent explicitement cette garantie dans leur docstring : ils ne
stockent que des métriques d'infrastructure.

---

## Architecture de l'agent

```
Heartbeat d'un nœud PME
        │
        ▼
ClusterMetricSample ──────────► série temporelle (6 h glissantes, 60 échantillons max)
        │
        ▼
build_features()  ──────────►  extraction : nb de nœuds, standbys en streaming,
        │                       min/max de réplication, écart de heartbeat,
        │                       bascules récentes
        ▼
   ┌────────────────────────────────────────┐
   │  Mistral AI (mistral-small-latest)     │  timeout 12 s
   │  clé absente ou appel en échec ?       │
   │            └──────────► repli local déterministe
   └────────────────────────────────────────┘
        │
        ▼
MonitorVerdict { risk_level, anomaly_score 0-100, summary,
                 recommendation, source, details }
        │
        ▼
AgentVerdict (persisté)  ──────►  page « Agent de supervision » + export PDF
```

**Le repli local n'est pas un bouchon.** C'est un moteur de règles complet : un état
`replication_down` / `no_primary` / `primary_offline` donne directement un verdict
`critique` à 90 ; une bascule récente ou un écart de heartbeat au-delà de 180 s donne
`surveiller` à 55. Chaque verdict est accompagné d'un `details` rédigé qui explique le
raisonnement. L'agent reste donc pleinement fonctionnel sans clé API — l'IA enrichit
l'analyse, elle n'en est pas la condition.

---

## Contenu de la branche

### Code de l'agent

```
devices/ai_monitor.py          414 l.  cœur : features, appel Mistral, replis, verdicts
devices/metrics_catalog.py     134 l.  traduit chaque métrique en langage clair + seuils
devices/host_monitor.py         78 l.  métriques du serveur éditeur
devices/relay_monitor.py        67 l.  santé du relais via /health, sans accès aux blobs
devices/models.py               +72 l. ClusterMetricSample, AgentVerdict
devices/views_web.py           +590 l. pages, analyses à la demande, export de rapport
```

Quatre migrations : `0007_clustermetricsample`, `0008_agentverdict`, `0009_agentverdict_details`,
`0010_device_epoch`.

### Routes ajoutées

| Route | Rôle |
|---|---|
| `/devices/agent/` | Page de supervision |
| `/devices/agent/live/` | Rafraîchissement des métriques |
| `/devices/agent/host-analyze/` | Analyse du serveur éditeur |
| `/devices/agent/relay-analyze/` | Analyse du relais |
| `/devices/agent/analyze-all/` | Analyse des trois acteurs |
| `/devices/agent/report/` · `report.pdf` | Export du rapport |
| `/devices/clusters/` · `<uuid>/ai-analyze/` | Vue des clusters, analyse ciblée |

### `metrics_catalog.py` — la traduction

Chaque métrique est décrite par un tuple `(libellé, unité, signification, seuil_surveiller,
seuil_critique)` :

```python
'mem_percent': ("Mémoire vive (RAM)", "%",
                "Taux d'occupation de la mémoire. Au-delà, le système ralentit.", 80, 92),
```

C'est ce qui permet à l'interface et au rapport exporté d'être lisibles par un administrateur
qui n'est pas ingénieur — un besoin réel côté PME.

---

## Autres apports de la branche

Au-delà de l'agent, 45 commits couvrent plusieurs chantiers :

**Fiabilité du cluster**
- Clôture (fencing) d'un primaire déchu via la timeline PostgreSQL
- Slot de réplication + plafond WAL + bascule manuelle à 2 nœuds
- Champ `epoch` porté sur le modèle `Device`

**Relais**
- Dépôt et supervision zero-knowledge des blobs par tenant
- Liste des blobs par tenant
- Correction de la syntaxe de route axum 0.7 (`:param`) pour `/api/blobs`
- Affichage de toutes les pattes réseau du relais (multi-segments)

**Déploiement**
- `RELAY_URL` auto-dérivé du segment PME (convention relais `.10`)
- `TENANT_ID` transmis au conteneur `ss-node`
- Compose des VM relais et nœuds via le registre d'images
- Pull forcé de la dernière image `ss-node` avant `up`

**Produit et interface**
- Identité **أمان (Amān)** sur les pages client et dans l'application
- Page Tarifs in-app, modèle économique open source
- Clé de récupération persistée + page d'administration dédiée
- Encart d'identifiants admin initiaux, effacé après la première connexion

**Livrables de soutenance** (`docs/livrables_soutenance/`)
- Rapport final (1 560 lignes), plan directeur (718 lignes), slides, version « CLAIR »
- Glossaire jury, mémo oral, fiche questions/réponses, script de démo chronométré
- Affiche A1 (HTML + PDF) avec ses images et son brief de conception
- CV, publication et profil LinkedIn

---

## Configuration

```bash
# .env — laisser vide active le repli local déterministe
MISTRAL_API_KEY=
```

```python
# config/settings.py
MISTRAL_API_KEY = config('MISTRAL_API_KEY', default='')
```

La clé n'est **jamais codée en dur**. Le modèle utilisé est `mistral-small-latest`, disponible
en offre gratuite ; le prompt est volontairement plafonné à 60 échantillons pour rester compact,
et le timeout est fixé à 12 secondes afin d'échouer vite vers le repli local.

**Sans clé, l'application fonctionne intégralement.** C'est un choix de conception : la
supervision ne doit pas dépendre d'un service externe.

---

## Points d'attention

- **Poids du dépôt.** La branche ajoute environ 19 000 lignes, dont une grande part de binaires :
  affiche PDF (6 Mo), images générées (2,3 Mo, 2,4 Mo, 1,7 Mo…), captures d'écran de tests.
  Les images de l'affiche sont présentes en double sous deux noms (`ChatGPT Image …png` et
  `images/afrique.png`, `images/hero.png`, `images/cle_dek.png`). À dédupliquer.
- **Dépendance implicite.** `ai_monitor.py` importe `certifi` directement, alors que celui-ci
  n'est présent que par transitivité via `requests` (bien déclaré, lui, en `2.32.5`).
  Cela fonctionne, mais un jour où `requests` cesserait d'en dépendre l'import casserait sans
  prévenir. Déclarer `certifi` explicitement dans `requirements.txt`.
- **Périmètre mixte.** Le code de l'agent et les livrables de soutenance cohabitent dans la même
  branche. Pour une fusion vers `main`, il serait plus sain de séparer les deux — le code d'un
  côté, `docs/livrables_soutenance/` de l'autre.
- **Statut de l'agent.** Les documents de soutenance présentent l'agent IA comme une
  **conception et une perspective**, non comme un composant déployé en production. Le README
  général de `main` reflète ce positionnement.

---

## Lancer et vérifier

```bash
git checkout feat/agent-supervision-ia

pip install -r requirements.txt
python manage.py migrate              # applique les migrations 0007 à 0010
python manage.py runserver 0.0.0.0:8000
```

Puis ouvrir `/devices/agent/` et lancer « analyser tout ». Sans `MISTRAL_API_KEY`, les verdicts
sont marqués `source = local` ; avec une clé valide, `source = mistral`.

---

## Conventions

Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Voir le [README.md général du projet](https://github.com/mpigajesse/saas-souverain/blob/main/README.md) sur la branche `main` pour l'architecture d'ensemble,
et [`CLAUDE.md`](CLAUDE.md) pour les décisions structurantes actées.
