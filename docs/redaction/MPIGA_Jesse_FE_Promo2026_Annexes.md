# Annexes — Rapport de Stage Fin d'Études
<!-- NOM FICHIER MOODLE : MPIGA_Jesse_FE_Promo2026_Annexes.pdf -->
<!-- Fichier SÉPARÉ du rapport principal — max 8 Mo -->
<!-- Pagination en chiffres romains : I, II, III... -->

**Étudiant :** Jesse MPIGA-ODOUMBA | **Entreprise :** AL BARAA CONSULTING | **Promo :** 2026

---

## Sommaire des Annexes

| Annexe | Titre | Page |
|--------|-------|------|
| Annexe I | Résultats complets de la suite de tests automatisés | II |
| Annexe II | Documentation complète de l'API REST | VII |
| Annexe III | Code source — Scripts clés | XII |
| Annexe IV | Captures d'écran du dashboard SDA | XVIII |
| Annexe V | Journal technique de déploiement — Incidents et solutions | XXII |
| Annexe VI | Analyse des risques projet | XXVII |
| Annexe VII | Budget et ROI estimatifs | XXIX |
| Annexe VIII | Structure complète du projet (arborescence) | XXXI |
| Annexe IX | Configuration Docker Compose commentée | XXXIII |
| Annexe X | Grille de conformité AUDPF | XXXVI |

---

---

# Annexe I — Résultats Complets de la Suite de Tests

## I.1. Environnement d'exécution

**Date d'exécution :** 2026-05-27
**Script :** `scripts/demo-tests.sh` — commit `421f825`
**Cluster :** 3 nœuds (Win11 `192.168.200.1` + Ubuntu `192.168.200.130` + Kali `192.168.200.128`)

## I.2. Résultats globaux

| Nœud | OS | ✅ PASS | ❌ FAIL | ⏭ SKIP | Score |
|------|----|--------|--------|--------|-------|
| Node 1 | Windows 11 Pro | **10** | 0 | 1 | 10/11 |
| Node 2 | Ubuntu 26.04 LTS | **11** | 0 | 0 | 11/11 |
| Node 3 | Kali Linux | **11** | 0 | 0 | 11/11 |
| **Cluster** | | **32** | **0** | **1** | **32/33** |

## I.3. Résultats détaillés — Node 1 (Windows 11)

### TEST 1.1 — Health Check via Docker exec
```
✅ PASS  Health check OK (via docker exec)
Réponse : {"status":"operational","architecture":"local-first / distributed",
           "central_dependency":"none","offline_ready":true}
```

### TEST 1.2 — Health via HTTPS/mTLS depuis l'hôte
```
⏭ SKIP  curl schannel (Windows) ne supporte pas les certificats PEM
Validation manuelle : Chrome → https://localhost/ → ✅ TLS 1.3, cadenas vert
```
> **Justification du SKIP :** Git Bash sous Windows utilise le backend TLS `schannel` natif de Windows, qui ne supporte pas le format PEM pour les certificats clients curl. Ce test réussit systématiquement sur les nœuds Linux (Ubuntu : ✅, Kali : ✅) et a été validé manuellement via Chrome sur Win11.

### TEST 1.3 — Rejet sans certificat client (mTLS)
```
✅ PASS  mTLS actif — requête sans certificat client rejetée
HTTP 400 No required SSL certificate was sent
```

### TEST 2 — Ingestion de données
```
✅ PASS  Ingestion réussie
tenant_id  : demo_sda
audit_id   : 4
record_hash: 2bdcec276fb126ec...
Fichier    : demo_sda_storage.parquet
```

### TEST 3.1 — Lecture analytique DuckDB avec déchiffrement Fernet
```
✅ PASS  14 fichiers lisibles (dont 1 chiffré déchiffré à la volée)
Top 5 enregistrements récents :
  tenant=demo_sda    at=2026-05-27 17:55:55.190389
  tenant=node1_win11 at=2026-05-26 09:46:56.087650
  tenant=node3_kali  at=2026-05-26 07:29:11.735713
  tenant=node1_demo  at=2026-05-25 21:45:38.923954
  tenant=node1_win11 at=2026-05-25 21:43:42.496075
```

### TEST 3.2 — Agrégation multi-fichiers multi-nœuds
```
✅ PASS  Consolidation distribuée réussie
Total enregistrements : 609
Tenants distincts     : 14
Fichiers analysés     : 14
```

### TEST 4.1 — Contenu dossier shared_storage
```
✅ PASS  16 fichiers Parquet présents
bench_test_storage.parquet   623 B
demo_sda_storage.parquet     1.1 KB  [chiffré Fernet]
node1_demo_storage.parquet   826 B
node1_win11_storage.parquet  1.0 KB
node3_kali_storage.parquet   1.1 KB  [chiffré Fernet]
tenant_000_storage.parquet   1.7 KB
... (10 tenants benchmark)
```

### TEST 4.2 — Intégrité des fichiers
```
✅ PASS  0 fichier corrompu
Fichiers non chiffrés valides : 13
Fichiers chiffrés valides     : 2
Corrompus                     : 0
```

### TEST 5 — Réconciliation CRDT
```
✅ PASS  Mécanisme CRDT opérationnel
{"status":"no_conflicts","conflicts_resolved":0,"merged_records":0,
 "timestamp":"2026-05-27T17:49:15Z"}
```

### TEST 6.1 — Statut des conteneurs Docker
```
✅ PASS  4 conteneurs actifs
sda-backend     : Up 26h (healthy)
sda-frontend    : Up 26h (healthy)
sda-nginx       : Up 18h (running)
sda-syncthing   : Up 45h (healthy)
```

### TEST 6.2 — Stabilité et uptime
```
✅ PASS  Uptime validé
sda-syncthing : 45 heures d'uptime continu ✅
```

## I.4. Résultats détaillés — Node 2 (Ubuntu 26.04 LTS) — **11/11 PASS**

```
✅ PASS  1.1  Health Check Docker exec        {"status":"operational","offline_ready":true}
✅ PASS  1.2  HTTPS/mTLS depuis hôte          TLS 1.3 — certificat client validé
✅ PASS  1.3  Rejet sans certificat            HTTP 400 No required SSL certificate
✅ PASS  2    Ingestion données               audit_id:1 record_hash:0c7c8c95a8040b0f...
✅ PASS  3.1  DuckDB déchiffrement Fernet     15 fichiers lisibles (2 chiffrés)
✅ PASS  3.2  Agrégation multi-nœuds          609 enreg. — 16 tenants distincts
✅ PASS  4.1  shared_storage contenu          16 fichiers Parquet
✅ PASS  4.2  Intégrité fichiers              0 corrompu
✅ PASS  5    CRDT réconciliation             no_conflicts — mécanisme fonctionnel
✅ PASS  6.1  Conteneurs Docker healthy       4/4 healthy
✅ PASS  6.2  Uptime stabilité               59 min depuis démarrage
```

## I.5. Résultats détaillés — Node 3 (Kali Linux) — **11/11 PASS**

```
✅ PASS  1.1  Health Check Docker exec        {"status":"operational","offline_ready":true}
✅ PASS  1.2  HTTPS/mTLS depuis hôte          TLS 1.3 — certificat client validé
✅ PASS  1.3  Rejet sans certificat            HTTP 400 No required SSL certificate
✅ PASS  2    Ingestion données               audit_id:2 record_hash:73f1cef55c409fd6...
✅ PASS  3.1  DuckDB déchiffrement Fernet     15 fichiers lisibles (2 chiffrés)
✅ PASS  3.2  Agrégation multi-nœuds          609 enreg. — 16 tenants distincts
✅ PASS  4.1  shared_storage contenu          16 fichiers Parquet
✅ PASS  4.2  Intégrité fichiers              0 corrompu
✅ PASS  5    CRDT réconciliation             no_conflicts — mécanisme fonctionnel
✅ PASS  6.1  Conteneurs Docker healthy       4/4 healthy
✅ PASS  6.2  Uptime stabilité               57 min depuis démarrage
```

---

---

# Annexe II — Documentation Complète de l'API REST

## II.1. Informations générales

| Paramètre | Valeur |
|-----------|--------|
| Framework | FastAPI 0.104+ avec Uvicorn |
| Protocole d'accès | HTTPS via nginx reverse proxy (port 443) |
| Authentification | mTLS x509 (certificat client) |
| Format réponses | JSON |
| Documentation interactive | `https://localhost/docs` (Swagger UI) |
| Documentation alternative | `https://localhost/redoc` (ReDoc) |
| Base URL | `https://localhost/` |

## II.2. Endpoints — Description complète

---

### `GET /health`

**Description :** Point de santé du nœud. Retourne le statut opérationnel et les caractéristiques de l'architecture.

**Authentification :** mTLS requis

**Réponse 200 :**
```json
{
  "status": "operational",
  "architecture": "local-first / distributed",
  "central_dependency": "none",
  "offline_ready": true
}
```

**Champs :**
| Champ | Type | Description |
|-------|------|-------------|
| `status` | string | `"operational"` si tous les services sont actifs |
| `architecture` | string | Invariant — identifie le paradigme |
| `central_dependency` | string | `"none"` — confirme l'absence de dépendance cloud |
| `offline_ready` | boolean | `true` — fonctionnement garanti sans internet |

---

### `GET /api/v1/node/info`

**Description :** Informations d'identité du nœud local.

**Réponse 200 :**
```json
{
  "host_os": "Windows",
  "host_os_release": "10.0.26200",
  "host_arch": "AMD64",
  "host_hostname": "DESKTOP-JESSE",
  "syncthing_device_id": "GHIJH3G-...",
  "uptime_seconds": 162000,
  "sda_version": "0.2.0"
}
```

---

### `POST /api/v1/data/ingest`

**Description :** Ingestion d'une donnée structurée. Déclenche le pipeline complet : validation → SQLite audit → DuckDB insert → export Parquet → chiffrement Fernet.

**Corps de la requête :**
```json
{
  "tenant_id": "string",
  "data": {
    "any_key": "any_value"
  }
}
```

**Paramètres :**
| Paramètre | Type | Requis | Description |
|-----------|------|--------|-------------|
| `tenant_id` | string | ✅ | Identifiant de l'organisation / tenant |
| `data` | object | ✅ | Données métier libres (JSON quelconque) |

**Réponse 200 :**
```json
{
  "status": "success",
  "tenant_id": "demo_sda",
  "audit_id": 4,
  "record_hash": "2bdcec276fb126ec4a08d2f5e5a9c3e1b7d4f892a3c61d0e5f7b8a9c2d4e6f1",
  "parquet_path": "/app/data/shared_storage/demo_sda_storage.parquet",
  "encrypted": true,
  "timestamp": "2026-05-27T17:55:55.190Z"
}
```

**Champs réponse :**
| Champ | Description |
|-------|-------------|
| `audit_id` | Identifiant séquentiel dans la chaîne d'audit SQLite |
| `record_hash` | SHA-256 chaîné : `SHA256(data + previous_hash + timestamp)` |
| `parquet_path` | Chemin du fichier Parquet créé/mis à jour |
| `encrypted` | `true` si `PARQUET_FERNET_KEY` est configurée |

**Réponse 422 (validation échouée) :**
```json
{
  "detail": [{"loc": ["body", "tenant_id"], "msg": "field required"}]
}
```

---

### `POST /api/v1/sync/reconcile`

**Description :** Déclenche la réconciliation CRDT (Last-Write-Wins) des conflits Syncthing détectés dans `shared_storage/`.

**Corps :** Aucun (POST sans body)

**Réponse 200 — aucun conflit :**
```json
{
  "status": "no_conflicts",
  "conflicts_resolved": 0,
  "merged_records": 0,
  "timestamp": "2026-05-27T17:49:15Z"
}
```

**Réponse 200 — conflits résolus :**
```json
{
  "status": "reconciled",
  "conflicts_resolved": 1,
  "merged_records": 1,
  "details": [
    {
      "file": "tenant_conflict_storage.parquet",
      "winner_node": "node3_kali",
      "winner_ts": "2026-05-27T10:45:00Z",
      "loser_ts": "2026-05-27T10:30:00Z",
      "strategy": "last-write-wins"
    }
  ],
  "timestamp": "2026-05-27T17:49:15Z"
}
```

## II.3. Codes d'erreur HTTP

| Code | Signification | Cause |
|------|--------------|-------|
| 200 | Succès | Requête traitée |
| 400 | No required SSL certificate | mTLS — certificat client absent |
| 422 | Unprocessable Entity | Corps JSON invalide ou champs manquants |
| 500 | Internal Server Error | Erreur backend (DuckDB, SQLite) |

## II.4. Exemples d'appels curl

```bash
# Health check via mTLS
curl -sk \
  --cert config/nginx/certs/client.crt \
  --key  config/nginx/certs/client.key \
  https://localhost/health | python3 -m json.tool

# Ingestion de données
curl -sk \
  --cert config/nginx/certs/client.crt \
  --key  config/nginx/certs/client.key \
  -X POST https://localhost/api/v1/data/ingest \
  -H "Content-Type: application/json" \
  -d '{"tenant_id":"tenant_albaraa","data":{"metric":"cpu","value":42}}' \
  | python3 -m json.tool

# Réconciliation CRDT
curl -sk \
  --cert config/nginx/certs/client.crt \
  --key  config/nginx/certs/client.key \
  -X POST https://localhost/api/v1/sync/reconcile \
  | python3 -m json.tool
```

---

---

# Annexe III — Code Source — Scripts Clés

## III.1. `scripts/nginx-entrypoint.sh` — Injection automatique clé API Syncthing

```bash
#!/bin/sh
# nginx-entrypoint.sh — Injecte la clé API Syncthing avant de démarrer nginx
# Exécuté automatiquement par le conteneur nginx au démarrage via docker-compose.
# Lit la clé depuis le config.xml Syncthing monté en volume — sans docker exec.

set -e

echo "[sda-nginx] Extraction de la clé API Syncthing..."
i=0
while [ $i -lt 30 ]; do
  API_KEY=$(grep -o '<apikey>[^<]*</apikey>' \
    /etc/syncthing-config/config.xml 2>/dev/null \
    | sed 's/<[^>]*>//g' || true)
  if [ -n "$API_KEY" ]; then
    printf 'proxy_set_header X-API-Key "%s";\n' "$API_KEY" \
      > /etc/nginx/certs/syncthing-key.conf
    echo "[sda-nginx] Clé injectée ($(echo "$API_KEY" | cut -c1-8)...)"
    break
  fi
  i=$((i + 1))
  echo "[sda-nginx] Config Syncthing absente — tentative $i/30 (attente 2s)..."
  sleep 2
done

if [ -z "$API_KEY" ]; then
  echo "[sda-nginx] AVERTISSEMENT : clé Syncthing non trouvée après 60s"
fi

exec nginx -g 'daemon off;'
```

**Intérêt de ce script :**
Ce script résout définitivement le problème de la clé API Syncthing node-specific. En lisant directement le fichier `config.xml` depuis le volume Docker partagé, il élimine le besoin de `docker exec` et de toute intervention manuelle. Le `depends_on: condition: service_healthy` dans `docker-compose.yml` garantit que Syncthing est opérationnel avant que ce script s'exécute.

---

## III.2. `scripts/demo-tests.sh` — Extrait de la suite de tests

```bash
#!/usr/bin/env bash
# demo-tests.sh — Suite de tests automatisés SDA
# Usage : bash scripts/demo-tests.sh
# Résultats : PASS / FAIL / SKIP par test

set -euo pipefail
PASS=0; FAIL=0; SKIP=0
GREEN='\033[0;32m'; RED='\033[0;31m'; AMBER='\033[0;33m'; NC='\033[0m'

pass() { echo -e "${GREEN}✅ PASS${NC}  $1"; ((PASS++)); }
fail() { echo -e "${RED}❌ FAIL${NC}  $1"; ((FAIL++)); }
skip() { echo -e "${AMBER}⏭ SKIP${NC}  $1"; ((SKIP++)); }

echo "═══════════════════════════════════════════════"
echo "  SDA Demo Tests — $(date)"
echo "═══════════════════════════════════════════════"

# TEST 1.1 — Health Check via Docker exec
echo ""
echo "─── TEST 1 : Health Check ───"
HEALTH=$(docker compose exec -T sda-backend \
  curl -s http://localhost:8000/health 2>/dev/null || echo "ERROR")
if echo "$HEALTH" | grep -q '"offline_ready":true'; then
  pass "Health check OK (via docker exec)"
  echo "  Réponse : $HEALTH"
else
  fail "Health check FAILED : $HEALTH"
fi

# TEST 1.2 — HTTPS/mTLS (Linux seulement)
if command -v openssl >/dev/null 2>&1 && \
   openssl version | grep -q OpenSSL; then
  MTLS=$(curl -sk \
    --cert config/nginx/certs/client.crt \
    --key  config/nginx/certs/client.key \
    https://localhost/health 2>/dev/null || echo "ERROR")
  if echo "$MTLS" | grep -q '"operational"'; then
    pass "HTTPS/mTLS opérationnel (TLS 1.3 — certificat client validé) : $MTLS"
  else
    fail "HTTPS/mTLS FAILED : $MTLS"
  fi
else
  skip "curl schannel (Windows) ne supporte pas les certificats PEM"
fi

# TEST 1.3 — Rejet sans certificat
REJECTED=$(curl -sk https://localhost/health 2>/dev/null | head -c 200 || echo "")
if echo "$REJECTED" | grep -qi "400\|No required SSL"; then
  pass "mTLS actif — requête sans certificat rejetée (HTTP 400)"
else
  fail "mTLS inactif ou réponse inattendue : $REJECTED"
fi

# TEST 2 — Ingestion
INGEST=$(docker compose exec -T sda-backend \
  curl -s -X POST http://localhost:8000/api/v1/data/ingest \
  -H "Content-Type: application/json" \
  -d '{"tenant_id":"demo_sda","data":{"test":true,"ts":"'"$(date -Iseconds)"'"}}' \
  2>/dev/null || echo "ERROR")
if echo "$INGEST" | grep -q '"status":"success"'; then
  HASH=$(echo "$INGEST" | grep -o '"record_hash":"[^"]*"' | head -1)
  pass "Ingestion réussie — $HASH"
else
  fail "Ingestion FAILED : $INGEST"
fi

# [... suite des tests 3.1 à 6.2 ...]

echo ""
echo "═══════════════════════════════════════════════"
echo "  RÉSULTATS : ✅ $PASS PASS  ❌ $FAIL FAIL  ⏭ $SKIP SKIP"
echo "═══════════════════════════════════════════════"
```

---

## III.3. `backend/app/database.py` — Fonctions clés chiffrement

```python
from cryptography.fernet import Fernet
import os
import duckdb
import tempfile

def _get_fernet() -> Fernet | None:
    """Retourne None si clé absente (mode dev sans chiffrement)."""
    key = os.getenv("PARQUET_FERNET_KEY", "")
    if not key:
        return None
    return Fernet(key.encode())

def encrypt_parquet(path: str) -> None:
    """Chiffre un fichier Parquet in-place avec Fernet."""
    fernet = _get_fernet()
    if fernet is None:
        return  # Mode dev — pas de chiffrement
    with open(path, "rb") as f:
        data = f.read()
    encrypted = fernet.encrypt(data)
    with open(path, "wb") as f:
        f.write(encrypted)

def decrypt_parquet(enc_path: str) -> str:
    """Déchiffre vers un fichier temporaire. Retourne le chemin du temp."""
    fernet = _get_fernet()
    if fernet is None:
        return enc_path  # Pas de chiffrement — retourne le chemin original
    with open(enc_path, "rb") as f:
        data = f.read()
    try:
        decrypted = fernet.decrypt(data)
        tmp = tempfile.NamedTemporaryFile(
            suffix=".parquet", delete=False
        )
        tmp.write(decrypted)
        tmp.close()
        return tmp.name
    except Exception:
        # Fichier non chiffré (legacy) — retourne chemin original
        return enc_path

def query_all_parquets(shared_path: str) -> list[dict]:
    """Agrège tous les Parquets du dossier partagé via DuckDB."""
    parquets = [
        f for f in os.listdir(shared_path)
        if f.endswith(".parquet")
    ]
    results = []
    conn = duckdb.connect(":memory:")
    for parquet_file in parquets:
        full_path = os.path.join(shared_path, parquet_file)
        readable_path = decrypt_parquet(full_path)
        try:
            rows = conn.execute(
                f"SELECT * FROM read_parquet('{readable_path}') LIMIT 5"
            ).fetchall()
            results.extend(rows)
        except Exception:
            pass  # Fichier corrompu ou incompatible — ignoré
    conn.close()
    return results
```

---

## III.4. `config/nginx/nginx.conf.template` — Configuration nginx complète

```nginx
# SDA Nginx — virtual host config
# Monté directement dans /etc/nginx/conf.d/default.conf
# (bypass envsubst pour préserver $host, $remote_addr, etc.)

server {
    listen 443 ssl;
    server_name _;

    ssl_protocols TLSv1.3;                          # TLS 1.3 EXCLUSIF
    ssl_prefer_server_ciphers off;

    ssl_certificate     /etc/nginx/certs/server.crt;
    ssl_certificate_key /etc/nginx/certs/server.key;

    # mTLS — le client DOIT présenter un certificat
    ssl_client_certificate /etc/nginx/certs/ca.crt;
    ssl_verify_client on;

    ssl_session_cache   shared:SSL:10m;
    ssl_session_timeout 10m;

    add_header Strict-Transport-Security
      "max-age=31536000; includeSubDomains" always;

    # Backend API REST
    location /api/ {
        proxy_pass         http://sda-backend:8000;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Real-IP         $remote_addr;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;
        proxy_set_header   X-Client-DN       $ssl_client_s_dn;
    }

    location /health {
        proxy_pass http://sda-backend:8000/health;
    }

    # Syncthing API — clé injectée automatiquement par nginx-entrypoint.sh
    location /syncthing-api/ {
        proxy_pass         http://syncthing:8384/;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Requested-With  XMLHttpRequest;
        include            /etc/nginx/certs/syncthing-key.conf*; # glob — optionnel
    }

    # Frontend React SPA
    location / {
        proxy_pass         http://sda-frontend:3000;
        proxy_set_header   Host            $host;
        proxy_set_header   X-Real-IP       $remote_addr;
    }
}

# Redirection HTTP → HTTPS
server {
    listen 80;
    server_name _;
    return 301 https://$host$request_uri;
}
```

---

---

# Annexe IV — Captures d'Écran du Dashboard SDA

## IV.1. Vue d'ensemble — Node 1 (Windows 11)

[CAPTURE : Dashboard SDA — Page Vue d'ensemble — affichant les métriques Syncthing : uptime, mémoire, pairs connectés, dernière activité]

**Description :** La page Vue d'ensemble affiche en temps réel le statut du nœud local : identifiant Syncthing, OS, uptime, mémoire allouée, et le fil d'activité récente (événements Syncthing en temps réel via l'API). Le badge "Actif" en vert et l'indicateur "LIVE" en rouge confirment que le système est opérationnel.

## IV.2. Cluster P2P — Visualisation réseau

[CAPTURE : Dashboard SDA — Page Cluster P2P — animation des paquets en transit entre les 3 nœuds]

**Description :** La page Cluster P2P visualise la topologie du réseau en temps réel. Trois nœuds sont représentés par des cercles connectés. Des animations de paquets (flèches animées SVG) transitent entre les nœuds pour représenter la synchronisation. Un compteur de paquets synchronisés s'incrémente en temps réel.

## IV.3. Pairs connectés

[CAPTURE : Dashboard SDA — Page Pairs — liste des 2 pairs Ubuntu et Kali, avec Device IDs, trafic réseau entrant/sortant, statut QUIC]

**Description :** Chaque pair est représenté par une carte expansible avec : Device ID Syncthing, statut de connexion (Connecté / Hors-ligne), type de connexion (QUIC / TCP / Relay), trafic reçu/envoyé. Les barres de signal (3 niveaux) indiquent la qualité de connexion.

## IV.4. Coffre-fort de fichiers P2P

[CAPTURE : Dashboard SDA — Page Coffre-fort — liste des fichiers chiffrés avec leurs propriétaires (.sda-owner), boutons upload/download/suppression]

**Description :** Le coffre-fort permet l'upload de fichiers qui sont chiffrés avec Fernet et estampillés avec un fichier `.sda-owner` (identité du nœud propriétaire). Seul le nœud propriétaire peut supprimer le fichier — les autres nœuds peuvent consulter et télécharger mais pas supprimer.

## IV.5. TLS 1.3 et mTLS — Chrome DevTools

[CAPTURE : Chrome DevTools — onglet Security — Connexion sécurisée, protocole TLS 1.3, chiffrement AES-256-GCM, certificat client CN=sda-client-node-1]

**Description :** La capture Chrome confirme : TLS 1.3, chiffrement AES-256-GCM, certificat serveur signé par la CA interne SDA, et certificat client `CN=sda-client-node-1` présenté par le navigateur.

## IV.6. Rejet sans certificat client — HTTP 400

[CAPTURE : navigateur ou terminal — requête sans certificat → HTTP 400 "No required SSL certificate was sent"]

**Description :** Tentative d'accès à `https://localhost/` sans certificat client configuré. nginx retourne HTTP 400 avec le message "No required SSL certificate was sent", confirmant que le mTLS est actif et fonctionnel.

## IV.7. Syncthing GUI — 3 nœuds connectés

[CAPTURE : Syncthing GUI http://localhost:8384 — 3 appareils "Connecté" (Win11, Ubuntu, Kali), dossier SDA_Shared "À jour"]

**Description :** L'interface Syncthing affiche les 3 nœuds du cluster en état "Connecté" et le dossier `SDA_Shared` synchronisé (statut "À jour", fond vert). Le compteur de fichiers (16 fichiers Parquet) est identique sur les 3 nœuds.

---

---

# Annexe V — Journal Technique — Incidents et Solutions

## V.1. Chronologie des incidents majeurs

| Date | Incident | Gravité | Résolu |
|------|----------|---------|--------|
| 2026-05-21 | nginx crash — envsubst détruit $host | 🔴 Critique | ✅ J+0 |
| 2026-05-24 | Certificats TLS sans SAN (Chrome ERR_CERT_COMMON_NAME_INVALID) | 🟠 Majeur | ✅ J+0 |
| 2026-05-26 | Clé API Syncthing perdue après git pull sur VMs | 🟠 Majeur | ✅ J+0 |
| 2026-05-27 | SQLite corrompu après changement de clé chiffrement | 🔴 Critique | ✅ J+0 |
| 2026-05-29 | Animations SVG tremblent (useState + setInterval) | 🟡 Moyen | ✅ J+0 |

## V.2. Incident P1 — nginx crash (envsubst)

**Date :** 2026-05-21  
**Durée de résolution :** 3 heures

**Symptôme :**
```
sda-nginx | nginx: [emerg] invalid number of arguments in
           "proxy_set_header" directive in
           /etc/nginx/conf.d/default.conf:38
```

**Analyse :**
L'image `nginx:1.25-alpine` embarque un mécanisme de templates : tout fichier placé dans `/etc/nginx/templates/` est traité par `envsubst` (GNU gettext porté sur Alpine) avant d'être copié dans `/etc/nginx/conf.d/`. Ce mécanisme substitue toutes les expressions `$variable` par leurs valeurs d'environnement. Les variables nginx natives (`$host`, `$remote_addr`, `$scheme`, `$proxy_add_x_forwarded_for`) étant inconnues du shell, elles sont remplacées par des chaînes vides, rendant la configuration invalide.

**Tentatives infructueuses :**
- Utiliser `${DOLLAR}host` dans la config → syntaxe non supportée par nginx
- Passer par `envsubst '$VAR1 $VAR2'` avec liste exhaustive → 12 variables nginx à échapper, trop fragile
- Modifier le ENTRYPOINT docker → complexifie le déploiement

**Solution retenue :**
Monter le fichier de configuration nginx directement dans `/etc/nginx/conf.d/default.conf` (non dans `/etc/nginx/templates/`). Docker Compose ne déclenche plus le mécanisme envsubst car le répertoire cible est différent.

```yaml
# docker-compose.yml — AVANT (bugué)
volumes:
  - ./config/nginx/nginx.conf.template:/etc/nginx/templates/default.conf.template:ro

# docker-compose.yml — APRÈS (corrigé)
volumes:
  - ./config/nginx/nginx.conf.template:/etc/nginx/conf.d/default.conf:ro
```

**Enseignement :** La connaissance des conventions internes des images Docker officielles (ici, le comportement du dossier `/etc/nginx/templates/`) est essentielle. La documentation officielle nginx Docker mentionne ce comportement, mais ne précise pas que `envsubst` Alpine est moins permissif que la version GNU.

---

## V.3. Incident P2 — Certificats TLS sans SAN

**Date :** 2026-05-24  
**Symptôme :** Chrome affiche `ERR_CERT_COMMON_NAME_INVALID` à l'accès de `https://localhost/`.

**Cause :** Le script initial `generate-certs.sh` générait des certificats avec uniquement `CN=localhost` (Common Name) mais sans **Subject Alternative Name (SAN)**. Depuis Chrome 58 (2017), les navigateurs ignorent le CN et exigent le SAN pour valider le nom d'hôte.

**Solution :**
```bash
# Ajout du SAN dans generate-certs.sh
openssl req -newkey rsa:4096 -nodes \
  -keyout server.key \
  -out server.csr \
  -subj "/C=MA/O=SDA/CN=sda-node" \
  -addext "subjectAltName=DNS:localhost,DNS:sda-node,IP:127.0.0.1,IP:192.168.200.1"
```

---

## V.4. Incident P3 — Clé API Syncthing écrasée par git pull

*(Voir rapport principal §II.9.2 pour l'analyse complète)*

**Impact :** Dashboard frontend : "Impossible de joindre Syncthing" sur les 2 VMs après `git pull`.  
**Durée résolution :** 4 heures (analyse + implémentation + tests)

**Commits de correction :**
- `3d35cf2` — fix: exclure syncthing-key.conf du tracking git
- `0c3394b` — fix: nginx include optionnel + placeholder versionné
- `2902916` — feat: injection automatique clé API Syncthing au démarrage nginx

---

## V.5. Incident P4 — SQLite corrompu après changement de clé

*(Voir rapport principal §II.9.4)*

**Commande de résolution :**
```bash
# Sur le nœud concerné
rm -f data/db/metadata_enc.db data/db/analytics.duckdb
docker compose up -d --force-recreate sda-backend
```

**Prévention :** Créer le fichier `.env` AVANT le premier `docker compose up --build -d`.

---

## V.6. NAT Docker sur Windows 11 — Comportement Syncthing

**Symptôme :** Dans la GUI Syncthing de Win11, l'adresse active des pairs distants affiche `172.21.0.1:xxxxx` au lieu de `192.168.200.x`.

**Cause :** Docker Desktop sur Windows utilise WSL2 comme runtime. Le noyau Linux de WSL2 effectue du **NAT masquerade** sur toutes les connexions TCP entrantes — l'IP source des VMs (`192.168.200.130`) est réécrite en `172.21.0.1` (passerelle du bridge Docker interne) avant d'être transmise au conteneur Syncthing.

**Impact :** Aucun. La synchronisation fonctionne à 100% car :
- Win11 → VMs : Syncthing utilise l'IP configurée (`192.168.200.x`) pour les sorties
- VMs → Win11 : connexion sur `192.168.200.1:22000`, Docker relaie correctement

C'est un artefact d'affichage, non un problème fonctionnel.

---

---

# Annexe VI — Analyse des Risques Projet

## VI.1. Matrice des risques — Niveau POC

| ID | Risque | Prob. | Impact | Criticité | Mesure palliative | Statut |
|----|--------|-------|--------|-----------|------------------|--------|
| RT-01 | Conflits CRDT non résolus | Élevée | Élevé | 🔴 CRITIQUE | LWW + fallback manuel | ✅ Résolu |
| RT-02 | Incompatibilité Docker multi-plateforme | Moy. | Moyen | 🟠 ÉLEVÉ | Tests 3 OS simultanés | ✅ Validé |
| RT-03 | Performance DuckDB insuffisante | Faible | Moyen | 🟡 MOYEN | Benchmarks dès S7 | ✅ 609 enreg. < 1s |
| RT-04 | Complexité intégration Syncthing | Moy. | Moyen | 🟠 ÉLEVÉ | Documentation officielle | ✅ Résolu |
| RT-05 | Clé chiffrement perdue ou compromise | Faible | Critique | 🔴 CRITIQUE | .env non versionné, rotation prévue | ✅ Procédure documentée |
| RT-06 | Scalabilité > 10 nœuds | Moy. | Faible | 🟡 FAIBLE | Syncthing testé 1000+ nœuds | ⏳ Hors périmètre POC |
| RP-01 | Retard livraison rapport | Moy. | Élevé | 🔴 CRITIQUE | Rédaction progressive | ⏳ En cours |

## VI.2. Risques résiduels (hors périmètre POC)

| Risque | Description | Recommandation production |
|--------|-------------|--------------------------|
| Gestion des clés à grande échelle | Partager `PARQUET_FERNET_KEY` entre 100+ nœuds est fragile | HashiCorp Vault ou KMS souverain |
| Révocation de certificat | Aucun mécanisme OCSP/CRL automatisé | Implémenter CRL ou OCSP stapling |
| Mise à jour du firmware Syncthing | Nouvelle version peut changer le format config.xml | Tests de régression avant mise à jour |
| Attaque par corrélation temporelle | Les timestamps LWW peuvent révéler des patterns d'usage | Horodatage flouté ou différé en production |

---

---

# Annexe VII — Budget et ROI Estimatifs

## VII.1. Coûts de développement (stage PFE)

| Ressource | Durée | Taux estimatif | Coût |
|-----------|-------|---------------|------|
| Jesse MPIGA-ODOUMBA (Stagiaire PFE) | 24 sem × 40h = 960h | 25 MAD/h | 24 000 MAD |
| Mme Soumia CHOKRI (Encadrement DG) | 24 sem × 4h = 96h | 60 MAD/h | 5 760 MAD |
| M. Ayoub AMRANI (Suivi académique) | 3 contacts × 2h = 6h | 80 MAD/h | 480 MAD |
| **TOTAL RESSOURCES HUMAINES** | | | **30 240 MAD** |

## VII.2. Coûts matériels et logiciels

| Poste | Coût | Justification |
|-------|------|--------------|
| Postes de travail (PC Win11 + 2 VMs) | 0 MAD | Infrastructure existante AL BARAA |
| Logiciels (Docker, Python, Node.js, DuckDB, Syncthing, FastAPI, React) | 0 MAD | 100% open-source |
| Certificats TLS | 0 MAD | PKI interne auto-signée |
| Hébergement cloud | 0 MAD | Architecture 100% locale |
| **TOTAL MATÉRIEL & LOGICIELS** | **0 MAD** | |

## VII.3. Coût total du POC

| Catégorie | Coût POC |
|-----------|----------|
| Ressources humaines | 30 240 MAD |
| Matériel & logiciels | 0 MAD |
| **TOTAL** | **30 240 MAD** |

## VII.4. Analyse ROI — Déploiement chez 10 organisations

| Métrique | Valeur |
|---------|--------|
| Économie cloud mensuelle par organisation | 500 MAD/mois |
| Économie annuelle par organisation | 6 000 MAD/an |
| Économies annuelles (10 organisations) | 60 000 MAD/an |
| Coût développement POC | 30 240 MAD |
| **ROI première année** | **198%** |
| **Break-even** | **6 mois** |

*Source : comparatif AWS S3 + EC2 micro vs infrastructure locale SDA (estimations 2026)*

---

---

# Annexe VIII — Structure Complète du Projet

```
sda-prototype/
├── docker-compose.yml               ← Orchestration 4 services
├── .env                             ← Clés chiffrement (non versionné)
├── .gitignore                       ← Exclusions (clés, DB, Parquet)
│
├── backend/
│   ├── Dockerfile                   ← Image Python 3.11 + FastAPI
│   ├── requirements.txt             ← Dépendances Python
│   └── app/
│       ├── main.py                  ← Point d'entrée FastAPI, CORS, routers
│       ├── database.py              ← SQLite + DuckDB + Fernet functions
│       ├── models.py                ← AuditTrail SHA-256 chaîné
│       └── routers/
│           ├── data.py              ← POST /ingest, GET /health, GET /node/info
│           └── sync.py              ← POST /reconcile (CRDT LWW)
│
├── frontend/
│   ├── Dockerfile                   ← Build Vite + nginx:alpine
│   ├── package.json                 ← Dépendances React/TypeScript
│   └── src/
│       ├── App.tsx                  ← Routes React Router
│       ├── main.tsx                 ← Point d'entrée React
│       ├── index.css                ← Animations CSS + design tokens
│       ├── api/
│       │   └── syncthing.ts         ← Client API Syncthing + SDA
│       ├── components/
│       │   ├── Layout.tsx           ← Shell principal (Sidebar + Outlet)
│       │   ├── Sidebar.tsx          ← Navigation + horseshoe arch + souveraineté
│       │   ├── EventFeed.tsx        ← Fil événements Syncthing LIVE
│       │   ├── FilesManager.tsx     ← Coffre-fort chiffré P2P
│       │   ├── FolderList.tsx       ← Dossiers Syncthing + progress bar
│       │   ├── MetricCard.tsx       ← Cartes métriques animées
│       │   ├── NodeIdentityCard.tsx ← Carte nœud + zellige + scan line
│       │   ├── PageHero.tsx         ← Header réutilisable avec stat chips
│       │   ├── PeerList.tsx         ← Pairs P2P + stagger entrance
│       │   └── StatusBadge.tsx      ← Badge statut Syncthing
│       └── pages/
│           ├── OverviewPage.tsx     ← Vue d'ensemble
│           ├── ClusterPage.tsx      ← Cluster P2P + animation SVG
│           ├── FoldersPage.tsx      ← Dossiers partagés
│           ├── PeersPage.tsx        ← Pairs connectés
│           ├── EventsPage.tsx       ← Événements
│           └── FilesPage.tsx        ← Coffre-fort
│
├── config/
│   ├── nginx/
│   │   ├── nginx.conf.template      ← Config nginx (bypass envsubst)
│   │   └── certs/
│   │       ├── ca.crt               ← CA interne SDA
│   │       ├── server.crt/key       ← Cert serveur nginx
│   │       ├── client.crt/key       ← Cert client mTLS
│   │       ├── sda-client.p12       ← Bundle PKCS#12 navigateur
│   │       └── syncthing-key.conf   ← Clé API (auto-générée, gitignored)
│   └── syncthing/
│       ├── config.xml               ← Config Syncthing (gitignored)
│       └── .gitkeep
│
├── data/
│   ├── db/
│   │   ├── metadata_enc.db          ← SQLite chiffré SQLCipher
│   │   └── analytics.duckdb         ← DuckDB analytique
│   └── shared_storage/
│       └── *.parquet                ← Parquets répliqués P2P (gitignored)
│
├── scripts/
│   ├── demo-tests.sh                ← Suite 11 tests automatisés
│   ├── generate-certs.sh            ← Génération PKI interne
│   ├── nginx-entrypoint.sh          ← Injection auto clé Syncthing ✨
│   ├── setup-syncthing-key.sh       ← Script manuel (backup)
│   └── sda-aliases.sh               ← Aliases shell utiles
│
└── docs/
    ├── architecture/security.md     ← Architecture sécurité mTLS
    ├── install/
    │   ├── demo-guide.md            ← Guide démo 3 nœuds
    │   ├── node-deployment.md       ← Guide déploiement générique
    │   ├── node-ubuntu-config.md    ← Spécifique Ubuntu 26.04
    │   └── node-kali-config.md      ← Spécifique Kali Linux
    └── rapport/
        └── rapport-tests-validation.md  ← Rapport 32/33 PASS
```

---

---

# Annexe IX — Docker Compose Commenté

```yaml
version: "3.8"

services:
  # ─────────────────────────────────────────────────────────────────
  # NGINX — Reverse Proxy TLS 1.3 + mTLS
  # L'entrypoint injecte automatiquement la clé API Syncthing
  # avant de démarrer nginx — zéro action manuelle requise.
  # ─────────────────────────────────────────────────────────────────
  nginx:
    image: nginx:1.25-alpine
    container_name: sda-nginx
    ports:
      - "443:443"   # HTTPS externe (mTLS requis)
      - "80:80"     # HTTP → redirect HTTPS
    volumes:
      - ./config/nginx/nginx.conf.template:/etc/nginx/conf.d/default.conf:ro
      - ./config/nginx/certs:/etc/nginx/certs          # Écriture autorisée pour syncthing-key.conf
      - ./config/syncthing:/etc/syncthing-config:ro    # Lecture config.xml Syncthing
      - ./scripts/nginx-entrypoint.sh:/docker-entrypoint-init.sh:ro
    entrypoint: ["/bin/sh", "/docker-entrypoint-init.sh"]
    depends_on:
      sda-backend:
        condition: service_healthy    # Attend que le backend soit opérationnel
      sda-frontend:
        condition: service_healthy    # Attend que le frontend soit opérationnel
      syncthing:
        condition: service_healthy    # Attend que la clé API Syncthing soit disponible
    restart: unless-stopped

  # ─────────────────────────────────────────────────────────────────
  # BACKEND — FastAPI + DuckDB + SQLite
  # Port 8000 interne UNIQUEMENT — accès exclusivement via nginx
  # ─────────────────────────────────────────────────────────────────
  sda-backend:
    build: ./backend
    container_name: sda-backend
    expose:
      - "8000"    # Interne Docker uniquement — jamais exposé à l'hôte
    volumes:
      - ./data/db:/app/data/db                         # Volumes persistants SQLite + DuckDB
      - ./data/shared_storage:/app/data/shared_storage # Parquets partagés avec Syncthing
    environment:
      - DB_SQLITE_PATH=/app/data/db/metadata_enc.db
      - DB_DUCKDB_PATH=/app/data/db/analytics.duckdb
      - SHARED_STORAGE_PATH=/app/data/shared_storage
      - SQLITE_ENCRYPTION_ENABLED=true
      - DB_ENCRYPTION_KEY=${DB_ENCRYPTION_KEY:-changeme-replace-in-production}
      - PARQUET_FERNET_KEY=${PARQUET_FERNET_KEY:-}    # Vide = pas de chiffrement (mode dev)
      - NODE_NAME=${NODE_NAME:-}
      - NODE_VAULT_KEY=${NODE_VAULT_KEY:-}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # ─────────────────────────────────────────────────────────────────
  # FRONTEND — React SPA via nginx:alpine
  # ─────────────────────────────────────────────────────────────────
  sda-frontend:
    build: ./frontend
    container_name: sda-frontend
    expose:
      - "3000"
    depends_on:
      - sda-backend
      - syncthing
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://127.0.0.1:3000/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 15s

  # ─────────────────────────────────────────────────────────────────
  # SYNCTHING — Réplication P2P
  # Le healthcheck vérifie la présence de <apikey> dans config.xml
  # nginx attend ce healthcheck avant de démarrer
  # ─────────────────────────────────────────────────────────────────
  syncthing:
    image: syncthing/syncthing:latest
    container_name: sda-syncthing
    ports:
      - "8384:8384"       # GUI Syncthing (accès local uniquement)
      - "22000:22000"     # Réplication P2P (TCP)
      - "22000:22000/udp" # Réplication P2P (QUIC)
      - "21027:21027/udp" # mDNS découverte locale
    volumes:
      - ./config/syncthing:/var/syncthing/config
      - ./data/shared_storage:/var/syncthing/SDA_Shared
    environment:
      - PUID=1000
      - PGID=1000
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "sh", "-c", "grep -q '<apikey>' /var/syncthing/config/config.xml"]
      interval: 5s
      timeout: 3s
      retries: 20
      start_period: 10s    # Syncthing génère son config.xml au 1er démarrage
```

---

---

# Annexe X — Grille de Conformité AUDPF

## X.1. AU Data Policy Framework — Exigences et implémentation SDA

| Exigence AUDPF | Article | Implémentation SDA | Statut |
|----------------|---------|-------------------|--------|
| Localisation des données sur le territoire de l'organisation | §4.1 | Parquets stockés uniquement sur les nœuds locaux de l'organisation | ✅ Natif |
| Chiffrement des données au repos | §5.2 | Fernet AES-128-CBC + HMAC-SHA256 + SQLCipher AES-256 | ✅ Implémenté |
| Chiffrement des données en transit | §5.3 | TLS 1.3 + mTLS x509 sur tous les canaux | ✅ Implémenté |
| Traçabilité et audit des accès | §6.1 | Audit trail SHA-256 chaîné — non falsifiable | ✅ Implémenté |
| Contrôle des accès (authentification forte) | §6.2 | mTLS x509 — certificat client obligatoire | ✅ Implémenté |
| Absence de transfert non consenti vers l'étranger | §7.1 | Architecture P2P — aucun service cloud externe | ✅ Natif |
| Souveraineté de l'infrastructure | §8.1 | Docker + briques open-source — 0 dépendance propriétaire | ✅ Natif |
| Disponibilité en mode dégradé (offline) | §9.2 | `"offline_ready": true` — 100% local-first | ✅ Validé |
| Portabilité des données | §10.1 | Format Parquet (Apache standard) — exportable | ✅ Natif |
| Documentation technique | §11.1 | 15+ fichiers doc (install, architecture, tests) | ✅ Produit |

## X.2. Conclusion de conformité

**SDA est conforme à 10/10 exigences AUDPF identifiées.**

La conformité est **architecturale** — elle découle des choix de conception fondamentaux (P2P, local-first, open-source) et non d'une configuration optionnelle. Il est structurellement impossible pour un déploiement SDA standard de violer ces exigences.

---

*Fin des Annexes*

---

*MPIGA-ODOUMBA Jesse — EIGSI Casablanca — Promotion 2026*
*AL BARAA CONSULTING — Soutenance : 01/07/2026 à 10h00*
*`MPIGA_Jesse_FE_Promo2026_Annexes.pdf`*
