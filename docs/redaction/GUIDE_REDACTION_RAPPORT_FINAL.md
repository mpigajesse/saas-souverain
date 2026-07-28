# Guide de Rédaction — Rapport Final SFE
## MPIGA-ODOUMBA Jesse — EIGSI Promo 2026 — AL BARAA CONSULTING

**Nom de fichier à déposer sur Moodle :**  
`MPIGA_Jesse_FE_Promo2026_Rapport_final.pdf`  
`MPIGA_Jesse_FE_Promo2026_Annexes.pdf` (annexes dans fichier séparé)

**SOUTENANCE : 01/07/2026 à 10h00 — présentiel EIGSI Casablanca**  
**Deadline dépôt Moodle :** **17/06/2026** (2 semaines avant soutenance)  
**Remettre à Mme CHOKRI pour validation : 10/06/2026** (elle signe la page de garde)  
**Volume :** 25–30 pages (hors remerciements, sommaire, annexes)  
**Taille max PDF :** 8 Mo

---

## 1. Règles de forme EIGSI (obligatoires)

### Police et mise en page
- Police : **Times New Roman** (ou équivalent serif), corps 12
- Notes de bas de page : corps 10
- Titres : entre 12 et 18, en gras autorisé
- Interligne simple, texte justifié, marges min. 2 cm
- Pas d'alinéa, espacement avant/après paragraphes (Word : Format > Paragraphe > Auto)
- Numérotation : **chiffres arabes** dans le texte, **chiffres romains** pour les annexes
- Subdivision max : 4 niveaux (ex : II.3.2.1)

### Orthographe — seuils pénalisants
| Fautes | Note structure |
|--------|---------------|
| 0 | 2/2 |
| ≤ 5 | 1,5/2 |
| ≤ 10 | 1/2 |
| ≤ 15 | 0,5/2 |
| > 15 | 0/2 — **STOP CORRECTION** si > 50 fautes |

> Faire relire par une personne extérieure. Utiliser le correcteur Word + Antidote.

### Usage de l'IA (obligatoire à déclarer)
Le guide EIGSI l'exige explicitement : **tout recours à une IA générative doit être mentionné en introduction**, en précisant la nature de l'aide apportée. L'EIGSI dispose d'outils anti-plagiat (Compilatio). À déclarer honnêtement.

### Figures, tableaux, illustrations
- Chaque figure/tableau est numéroté et légendé
- Listés dans une "Table des illustrations" après le sommaire
- Appelés depuis le texte (ex : "voir Figure 3")
- Pas trop de captures d'écran — préférer des diagrammes, schémas, courbes

---

## 2. Structure imposée du rapport

### Page de couverture
```
[Logo EIGSI]          [Logo AL BARAA CONSULTING]

          RAPPORT DE STAGE — FIN D'ÉTUDES
          EIGSI Casablanca — Promotion 2026

Titre : Conception et Implémentation d'une Architecture
        Coffre-Fort Data P2P Souveraine

Date de début : 19 février 2026
Date de fin   : [date réelle fin de stage]

┌─────────────────┬──────────────────────┬─────────────────┐
│ Encadrant       │ Tuteur EIGSI         │ Étudiant        │
│ Entreprise      │                      │                 │
├─────────────────┼──────────────────────┼─────────────────┤
│ Soumia CHOKRI   │ [Nom tuteur EIGSI]   │ Jesse MPIGA-    │
│ Directrice      │ Enseignant IABD      │ ODOUMBA         │
│ Générale        │ +212 XXX XXX XXX     │ +XXX XXX XXX    │
│ chokri.soumaya  │ email@eigsica.ma     │ j.mpiga.26@     │
│ 90@gmail.com    │                      │ eigsica.ma      │
└─────────────────┴──────────────────────┴─────────────────┘

Année académique : 2025–2026
```

### Pages liminaires (dans l'ordre)
1. Page de couverture
2. Résumé + mots-clés (en français)
3. Remerciements (sobre, nommer : Mme CHOKRI, tuteur EIGSI, équipe)
4. Table des matières (automatique Word, avec numéros de page)
5. Table des illustrations (figures + tableaux)
6. Liste des abréviations

---

## 3. Plan détaillé du rapport

### INTRODUCTION GÉNÉRALE (1–2 pages)
Rédiger **en dernier** après tout le reste.

Doit contenir :
- Contexte global : souveraineté numérique en Afrique, AUDPF, dépendance cloud
- La problématique centrale : *"Comment garantir la souveraineté, la sécurité et la résilience des données dans un environnement africain, sans cloud centralisé étranger ?"*
- Présentation de l'entreprise d'accueil (une phrase) et du cadre du stage
- Annonce du plan du rapport (3 parties)
- Mention du recours à l'IA

---

### PARTIE I — Présentation de l'Entreprise et Contexte du Stage (4–5 pages)

**Source principale :** rapport du camarade Elvis-Theo AKIEMEOYONO (même entreprise — reprendre les infos AL BARAA CONSULTING en les reformulant)

#### I.1. AL BARAA CONSULTING — Présentation et positionnement
- Cabinet de conseil et d'ingénierie numérique
- Fondé en mars 2017, SARL AU, capital 100 000 MAD
- Basé à Casablanca — Résidence Al Amane GH31, Ain Sebaa
- Dirigé par Mme Soumia CHOKRI, Directrice Générale
- Missions : développement logiciel, architecture SI, transformation numérique
- Clientèle : publique et privée (BtoB)
- Structure à taille humaine : réactivité, responsabilisation forte
- Positionnement : ingénierie numérique souveraine, conseil en systèmes distribués

> Ajouter : organigramme simplifié, logo, adresse complète, domaine d'activité NAF/APE

#### I.2. Contexte du Stage et Enjeux
- Spécialité : Big Data & IA — EIGSI Casablanca Promo 2026
- Durée : 19 février → 6 août 2026 (24 semaines)
- Mission confiée : conception et implémentation d'un prototype d'architecture décentralisée souveraine
- Enjeu stratégique pour AL BARAA : démontrer la faisabilité d'un BaaS local open-source

#### I.3. Analyse Fonctionnelle
- **Bête à cornes** : à qui profite-t-il ? sur quoi agit-il ? dans quel but ?
- **Diagramme pieuvre** : fonctions principales (FP1, FP2) et contraintes (FC1–FC5)
- **Diagramme FAST** : décomposition fonctionnelle (stockage local → synchro P2P → sécurité)

> Ces éléments existent déjà dans le Plan Directeur — les adapter avec des figures propres

#### I.4. État de l'Art et Positionnement Technologique
- Benchmark : IPFS/Filecoin vs Solid Pods vs **SDA (Brique Universelle)**
- 3 différenciateurs : OLAP natif, offline-first radical, souveraineté infrastructurelle
- Conformité AUDPF (African Union Data Policy Framework)

#### I.5. Objectifs SMART de la Mission
Reprendre les 5 objectifs du Plan Directeur en les reformulant brièvement.

---

### PARTIE II — Bilan Technique (15–18 pages)

#### II.1. Méthodologie de Travail
- Approche "orchestration de briques open-source" vs "from scratch"
- Démarche itérative : prototype → validation → correction → POC complet
- Outils : Git/GitHub, Docker Compose, VSCode, Claude AI (à mentionner)

#### II.2. Architecture Technique Globale
- **Diagramme d'architecture C4** (niveau Conteneur) : 4 services Docker
- 3 couches : Nginx mTLS / FastAPI / DuckDB+Syncthing
- Paradigme Code-to-Data : les données ne quittent jamais le nœud local
- Table des ports et services

#### II.3. Module Stockage Local
- FastAPI + Uvicorn : API REST, endpoints `/health`, `/api/v1/data/ingest`, `/api/v1/sync/reconcile`
- DuckDB (OLAP) : export Parquet, requêtes analytiques multi-nœuds
- SQLite (ACID) : métadonnées, audit trail
- Audit trail SHA-256 chaîné (blockchain-style) : `record_hash` = SHA256(données + previous_hash)
- Performance validée : 609 enregistrements, 16 tenants, < 1s

#### II.4. Module Synchronisation P2P
- Syncthing (protocole BEP/TLS 1.3) : réplication automatique des fichiers Parquet
- mDNS : découverte des nœuds sans configuration réseau complexe
- CRDT Last-Write-Wins : résolution de conflits sur timestamp
- Validation : 3 nœuds (Win11 + Ubuntu 26.04 + Kali Linux), 0 perte de données

#### II.5. Sécurité
- **Transit** : TLS 1.3 uniquement (TLS 1.2 rejeté) + mTLS x509 (certificat client obligatoire)
- **Au repos** : Fernet AES-128-CBC + HMAC-SHA256 sur fichiers Parquet
- **Architecture** : Nginx reverse proxy avec CA interne SDA, clé API Syncthing injectée automatiquement
- Rejet HTTP 400 sans certificat client — validé sur 3 nœuds

#### II.6. Interface Utilisateur (Frontend)
- React + TypeScript + Vite, servi via Nginx Alpine
- Dashboard : Vue d'ensemble, Cluster P2P, Dossiers, Pairs, Événements, Coffre-fort
- Design "Moroccan Dark Tech" : identité africaine et souveraine
- Syncthing API proxifiée via nginx (clé injectée automatiquement au démarrage)

#### II.7. Déploiement et Infrastructure
- Docker Compose : 4 services orchestrés (`sda-nginx`, `sda-backend`, `sda-frontend`, `sda-syncthing`)
- `docker compose up --build -d` → cluster opérationnel en ~15 min (cible : < 30 min ✅)
- `nginx-entrypoint.sh` : injection automatique de la clé API Syncthing au démarrage nginx
- `depends_on: condition: service_healthy` : orchestration par santé de service

#### II.8. Tests et Validation
- Suite automatisée : `scripts/demo-tests.sh` — 11 tests, 6 catégories
- Résultats cluster 3 nœuds : **32/33 PASS** (1 SKIP Win11 — limitation TLS schannel, validé navigateur)
- Win11 : 10/11 PASS | Ubuntu : 11/11 PASS | Kali : 11/11 PASS
- Critères POC tous atteints (tableau des critères)

#### II.9. Difficultés Rencontrées et Solutions
*(extraire du journal technique — top 3 à 5)*

| Problème | Cause | Solution |
|----------|-------|----------|
| nginx crash au démarrage | `envsubst` détruisait les variables nginx `$host`, `$remote_addr` | Monte `nginx.conf.template` directement en `/etc/nginx/conf.d/default.conf` (bypass envsubst) |
| "Impossible de joindre Syncthing" sur les VMs | Clé API node-specific écrasée par `git pull` | `syncthing-key.conf` exclu du git + injection auto via `nginx-entrypoint.sh` |
| Tremblement animations SVG sur /cluster | `useState` + `setInterval` = re-render React → restart `<animateMotion>` | Remplacé par `useRef` + mise à jour DOM directe (sans re-render) |
| SQLite corrompu après changement de clé | `metadata_enc.db` créée avec clé par défaut, incompatible après `.env` | Supprimer la DB, redémarrer avec la bonne clé |
| Adresse Syncthing affiche 172.21.0.x sur Win11 | NAT masquerade Docker/WSL2 | Comportement normal — pas correctif, documenté |

#### II.10. Conclusion Technique
- Tous les critères de succès POC atteints
- Architecture extensible : ajouter des nœuds sans reconfiguration centrale
- Perspective : passage POC → production (10+ nœuds, gRPC, chiffrement SQLCipher)

---

### PARTIE III — Bilan de l'Expérience (3–4 pages)

#### III.1. Compétences Mobilisées et Acquises
- Big Data & IA : DuckDB OLAP, Parquet, systèmes distribués
- Cybersécurité : TLS 1.3, mTLS x509, chiffrement symétrique Fernet
- DevOps : Docker Compose, CI/CD, orchestration multi-conteneurs
- Développement full-stack : FastAPI (Python), React (TypeScript), nginx
- Gestion de projet : WBS, RACI, planning SMART, documentation technique

#### III.2. Valeur Ajoutée pour AL BARAA CONSULTING
- Prototype fonctionnel livré et validé — POC démontrable au jury
- Documentation technique complète réutilisable pour futurs clients
- Positionnement innovant : alternative souveraine au cloud pour PME africaines
- ROI estimatif : 184% dès la 1ère année (10 organisations × 6 000 MAD/an d'économies)

#### III.3. Réflexion Personnelle et Projet Professionnel
- Première expérience de déploiement multi-nœuds en conditions réelles
- Développement de l'autonomie : résolution de bugs critiques en production
- Prise de conscience des enjeux de souveraineté numérique en Afrique
- Perspectives professionnelles : ingénierie systèmes distribués, cybersécurité, BaaS souverain

---

### CONCLUSION GÉNÉRALE (1 page)

**Partie technique :**
- Objectifs SMART : tous atteints
- 32/33 tests automatisés PASS — cluster 3 nœuds opérationnel
- Architecture scalable, open-source, 100% offline-first

**Partie personnelle :**
- Ce projet répond à une problématique réelle et urgente pour le continent africain
- Il m'a permis de mobiliser l'ensemble de mes compétences EIGSI en situation réelle
- Perspectives : contribuer au développement de solutions numériques souveraines adaptées au contexte africain

---

## 4. Annexes (fichier séparé)

À inclure dans `MPIGA_Jesse_FE_Promo2026_Annexes.pdf` :

| Annexe | Contenu |
|--------|---------|
| Annexe I | Résultats complets demo-tests.sh (32/33 PASS) |
| Annexe II | Captures d'écran dashboard (7 scénarios jury) |
| Annexe III | Schéma architecture Docker Compose complet |
| Annexe IV | API Swagger — endpoints documentés |
| Annexe V | Code `nginx-entrypoint.sh` + `demo-tests.sh` |
| Annexe VI | Journal technique de déploiement (extraits clés) |
| Annexe VII | Benchmark IPFS vs Solid vs SDA (tableau complet) |

---

## 5. Grille d'évaluation EIGSI — rapport final (20%)

Le tuteur EIGSI note 5 critères (note 0-4 chacun) :

| Critère | Ce qui est évalué | Comment optimiser |
|---------|------------------|-------------------|
| **① Structure du rapport** | Présentation entreprise, plan, visuels, orthographe, style, respect du standard | Soigner la couverture, le sommaire auto, les figures légendées, 0 faute |
| **② Qualité du contenu technique** | Problématique, analyse scientifique, méthode, solution et résultats | Justifier chaque choix technique, comparer aux alternatives |
| **③ Synthèse des travaux** | Pertinence conclusions, perspectives, représentativité du travail fourni | Relier les résultats aux objectifs SMART initiaux |
| **④ Valeur ajoutée pour l'Entreprise** | Conformité cahier des charges, exploitation des résultats | Quantifier : 32/33 PASS, 15 min déploiement, 45h uptime |
| **⑤ (Transversal)** | Démarche méthodologique, références bibliographiques, réflexion sur résultats | Citer des sources académiques (CRDT, BEP, TLS 1.3) |

**Défauts récurrents à éviter (signalés par l'EIGSI chaque année) :**
- Trop descriptif — **analyser, ne pas raconter**
- Pas assez de diagrammes, modèles, courbes — **montrer, pas juste écrire**
- Pas de réflexion sur l'interprétation des résultats — **expliquer pourquoi 32/33 PASS signifie quoi**
- Peu de références bibliographiques — **citer : CRDT (Shapiro 2011), BEP Syncthing, AUDPF, FastAPI docs**
- Peu de réflexion sur la valeur ajoutée ingénieur — **différencier : qu'est-ce qu'un ingénieur apporte qu'un technicien n'aurait pas apporté ?**

---

## 6. Points spécifiques à ne pas oublier

### Mention IA obligatoire (introduction)
```
Ce rapport a été rédigé avec l'appui de Claude AI (Anthropic) pour :
- la structuration de certains paragraphes techniques
- la reformulation d'extraits de documentation
- la génération de tableaux de synthèse
L'analyse, la réflexion critique et les décisions techniques sont le fruit
du travail de l'auteur.
```

### Enjeux DDRS (Développement Durable & Responsabilité Sociétale)
L'EIGSI évalue la prise en compte des enjeux DDRS dans la structure du rapport. À intégrer dans :
- **Partie I** : enjeu de souveraineté numérique = enjeu sociétal africain, conformité AUDPF
- **Partie II** : 100% open-source = 0 MAD de licences, zéro dépendance à infrastructure étrangère
- **Conclusion** : résilience et autonomie numérique des organisations africaines

### Validation par Mme CHOKRI
- Remettre le rapport à Mme CHOKRI **3 semaines avant la soutenance** (soit ~mi-juillet)
- Elle doit **signer la page de garde** pour valider
- Déposer sur Moodle **2 semaines avant** la soutenance

---

## 7. Sources à citer dans le rapport

| Référence | Usage |
|-----------|-------|
| AU Data Policy Framework (AUDPF) — African Union, déc. 2025 | Contexte réglementaire |
| Shapiro et al., "Conflict-free Replicated Data Types", 2011 | Justification CRDT |
| Syncthing BEP Protocol — syncthing.net | Protocole réplication P2P |
| DuckDB Documentation — duckdb.org | Justification OLAP |
| RFC 8446 — TLS 1.3 | Justification sécurité |
| FastAPI Documentation — fastapi.tiangolo.com | Justification API |
| Docker Documentation — docs.docker.com | Justification conteneurisation |
| IPFS Documentation — docs.ipfs.tech | Benchmark état de l'art |

---

*Document de référence — à conserver pendant toute la rédaction*  
*Mis à jour le 2026-05-30 — MPIGA-ODOUMBA Jesse — EIGSI × AL BARAA CONSULTING*
