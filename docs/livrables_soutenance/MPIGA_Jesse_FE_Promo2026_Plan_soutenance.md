# Plan de Soutenance — Expérience Professionnelle de Fin d'Études
<!-- NOM FICHIER MOODLE : MPIGA_Jesse_FE_Promo2026_Plan_soutenance.pdf -->
<!-- DEADLINE DEPOT MOODLE : 24/06/2026 (1 semaine avant soutenance) -->

---

**Étudiant :** Jesse MPIGA-ODOUMBA
**Entreprise :** AL BARAA CONSULTING
**Tuteur entreprise :** Mme Soumia CHOKRI
**Tuteur EIGSI :** M. Ayoub AMRANI
**Date de soutenance :** 01 juillet 2026 à 10h00
**Lieu :** EIGSI Casablanca — en présentiel

**Sujet :** Conception et Implémentation d'un **Framework SaaS Souverain** pour Logiciels Métier Distribués

---

## Sommaire de la Présentation

| # | Partie | Durée | Slides |
|---|--------|-------|--------|
| 1 | Présentation de l'entreprise et contexte | 2 min | 1–3 |
| 2 | Problématique : la souveraineté des données métier | 3 min | 4 |
| 3 | La solution : trois acteurs + modèle économique | 3 min | 5–6bis |
| 4 | Architecture & garantie zero-knowledge | 4 min | 7–9 |
| 5 | Le cœur technique : réplication & résilience | 2 min | 8 |
| 6 | Démonstration live — cluster PME 2 nœuds | 5 min | 10 |
| 7 | Résultats et validation — 6/6 scénarios | 2 min | 11 |
| 8 | **Supervision intelligente — agent IA** | 3 min | 12 |
| 9 | Sécurité & difficultés (split-brain) | 4 min | 13–14 |
| 10 | Bilan, compétences & perspectives | 2 min | 15–16 |
| 11 | Conclusion | 1 min | 17 |
| — | **TOTAL** | **~31 min** | **17 slides** |

---

## Détail par Partie

### Partie 1 — AL BARAA CONSULTING et Contexte (2 min — Slides 1–3)

**Slide 1 — Page de garde**
- Logos EIGSI + AL BARAA CONSULTING
- Titre : "Framework SaaS Souverain pour Logiciels Métier Distribués"
- Nom, Promo 2026, encadrante entreprise, tuteur EIGSI, date

**Slide 3 — Entreprise et mission**
- AL BARAA CONSULTING : cabinet de conseil numérique, Casablanca, fondé 2017
- Mission : concevoir un framework permettant de vendre un logiciel métier en SaaS sans accès aux données clients
- Durée : 24 semaines (19 février → 01 juillet 2026)

*Message clé :* AL BARAA m'a confié la responsabilité complète d'un projet stratégique — une mission d'ingénieur, pas d'exécution.

---

### Partie 2 — Problématique et Enjeux (3 min — Slide 4)

**Slide 4 — Le dilemme de l'éditeur**
- Vendre en SaaS = héberger les données = **voir** les données métier du client
- Alternative (logiciel installé) = perte des licences, mises à jour, support
- AUDPF (AU Data Policy Framework, déc. 2025) : exige la souveraineté des données africaines
- Question centrale : *Comment vendre un logiciel métier en SaaS complet tout en étant cryptographiquement incapable de lire les données du client ?*

*Message clé :* Le problème est autant commercial que technique — concilier modèle SaaS et souveraineté.

**Objectifs de la mission (SMART) — annoncés en transition slide 4 → 5 :**
- **O1** — Concevoir une architecture séparant gestion commerciale (éditeur) et données métier (client), avec garantie zero-knowledge.
- **O2** — Prouver sur banc réel multi-OS la réplication PostgreSQL primaire/standby et la résilience à la panne d'un nœud.
- **O3** — Valider la cohérence des données métier (transactions, contraintes, concurrence) répliquées sans divergence.
- **O4** — Livrer un packaging déployable (image Docker) et un SaaS éditeur gérant comptes, licences et suivi du parc.
- **O5** — Implémenter une supervision intelligente (agent IA) qui analyse en temps réel les métriques d'infrastructure des **3 acteurs** (serveur éditeur, relais zero-knowledge, clusters PME), produit des diagnostics et des rapports exploitables — sans jamais accéder aux données chiffrées.

---

### Partie 3 — La Solution : Trois Acteurs (3 min — Slides 5–6)

**Slide 5 — Architecture à trois acteurs**
- SaaS éditeur (Django) : comptes tenants, licences, suivi du parc — voit compte/licence seulement
- Relais zero-knowledge : stocke des blobs chiffrés opaques — ne voit **rien**
- Cluster PME : exécute le logiciel, détient les données **en clair** dans son périmètre

**Slide 6 — Positionnement**
- vs SaaS cloud classique (expose les données) · vs logiciel installé (perd le SaaS)
- 3 différenciateurs : SaaS complet sans accès données / résilience BDD / souveraineté cryptographique

**Slide 6 bis — Modèle économique (prix des licences)**
- Plans Starter / Pro / Enterprise par sièges (postes) — grille **dégressive** par volume
- Estimation : **149 / 119 / 89 DH** par poste/mois vs **250–1 100 DH** pour les SaaS classiques (Odoo, Sage, Zoho, Dynamics)
- Exemple PME 10 postes : **≈ 14 280 DH/an** vs **≈ 48 000 DH/an** → **~70 % d'économie**
- Logique : l'éditeur n'héberge pas les données (elles restent chez la PME) → coût marginal faible → prix par poste plus bas, marge préservée

*Message clé :* On découpe le rôle en trois pour séparer ce qui peut être vu de ce qui ne doit jamais l'être — et ce découpage rend le produit **moins cher** pour la PME, pas plus cher : la souveraineté n'est pas un surcoût.

---

### Partie 4 — Architecture & Zero-Knowledge (4 min — Slides 7–9)

**Slide 7 — Hiérarchie de clés**
- DEK (clé symétrique unique par entreprise) chiffre données + journal CBOR
- Emballée par sealed box (X25519) pour chaque appareil, et sous code de récupération (Argon2id) sur le relais
- Crypto via libsodium uniquement — aucune primitive réinventée ; cœur en Rust (UniFFI)

**Slide 9 — Enrôlement & récupération de sinistre**
- Enrôlement : partage de la DEK chiffrée vers un nouvel appareil (jeton à usage unique)
- Récupération : l'éditeur restitue un blob chiffré que seul le client peut ouvrir (code de récupération)

*Message clé :* La souveraineté est une propriété cryptographique, pas une promesse contractuelle.

---

### Partie 5 — Cœur Technique : Réplication (2 min — Slide 8)

**Slide 8 — Cluster PME PostgreSQL primaire/standby**
- Réplication streaming WAL (< 1 s), journal append-only CBOR chiffré
- `standby.signal` : le standby refuse d'écrire → ne peut pas diverger
- Failover : 2 nœuds = bascule manuelle · ≥ 3 nœuds = quorum automatique
- Packaging Docker multi-arch, annonce des nœuds au relais (pas de mDNS)

*Message clé :* Je n'ai réinventé ni la crypto ni le consensus — j'ai orchestré des briques éprouvées (PostgreSQL).

---

### Partie 6 — Démonstration Live (5 min — Slide 10)

**Slide 10 — Démo en direct (cluster 2 nœuds, tenant MPJ)**
1. Alice (employé) crée 2 articles → enregistrés sur le primaire Kali
2. Bob (employé) fait une sortie de stock → base partagée
3. Requête SQL sur le standby Ubuntu → données répliquées en < 1 s
4. Tableau de bord SaaS → « ✓ Cluster sain »
5. Coupure du standby → SaaS détecte « Réplication interrompue »
6. Redémarrage → rattrapage WAL automatique + retour « sain »
7. **Tableau de bord** : KPI temps réel (nœuds en ligne, réplication active, occupation licences, santé du serveur éditeur)
8. **Agent IA** : clic sur « Analyser » → verdict (risque + score + diagnostic) sur le cluster en panne, puis « Diagnostic IA serveur » et « Diagnostic IA relais » (les 3 acteurs) · téléchargement du rapport PDF

*Message clé :* Deux machines réelles, réplication transparente, un SaaS qui ne ment jamais sur l'état réel — et un agent IA qui supervise les 3 acteurs en direct, sans jamais lire de donnée chiffrée.

---

### Partie 7 — Résultats et Validation (2 min — Slide 11)

**Slide 11 — 6/6 scénarios validés**
- Réplication, base partagée 2 employés, cohérence ACID + alerte seuil
- Unicité (contrainte UNIQUE), concurrence MVCC, panne/rattrapage WAL
- 0 échec, réplication streaming confirmée par `pg_stat_replication`

*Message clé :* Pas une démo de laboratoire — une campagne de test structurée sur banc multi-OS.

---

### Partie 8 — Supervision Intelligente : Agent IA (3 min — Slide 12)

**Slide 12 — L'angle Big Data & IA** *(implémenté et démontrable)*
- Les métriques sont collectées en base (`ClusterMetricSample`) : heartbeats (~60 s), streaming_standby_count, WAL lag, failover_count
- **Agent implémenté** : un agent LLM (Mistral, free tier) analyse ces métriques et rend un verdict structuré — niveau de risque (sain/surveiller/critique), score d'anomalie /100, diagnostic et recommandation
- **Supervision des 3 acteurs** : (1) serveur éditeur SaaS (CPU, RAM, disque, base) · (2) **relais zero-knowledge** (disponibilité, uptime, nombre de blobs — jamais le contenu) · (3) clusters PME (réplication, heartbeats, failover) — page temps réel animée
- **Robustesse** : repli local déterministe si l'IA est indisponible → démontrable même hors-ligne · zero-knowledge respecté jusque dans la supervision (métriques d'infra / métadonnées uniquement, aucune donnée métier ni blob chiffré)
- **Livrables** : rapport PDF détaillé (méthodologie + raisonnement de l'agent par métrique) + KPI au tableau de bord
- **Perspective restante (honnête)** : remplacer/compléter le raisonnement LLM par un modèle non supervisé entraîné (Isolation Forest / z-score glissant) sur de longues séries → **prédiction** de panne avant franchissement de seuil

*Message clé :* L'agent supervise toute l'architecture — éditeur, relais, clusters — en respectant la souveraineté jusque dans le monitoring : sur le relais, il ne lit que la santé, jamais le contenu chiffré. L'étape suivante est un modèle prédictif entraîné sur l'historique accumulé.

---

### Partie 9 — Sécurité & Difficultés (4 min — Slides 13–14)

**Slide 13 — Défense en profondeur**
- Zero-knowledge (relais sans clé) · at-rest XChaCha20-Poly1305 · enrôlement sealed box
- Fencing par jeton d'époque · identité parc par UUID d'installation (pas la MAC)

**Slide 14 — Diagnostic d'un split-brain (incident réel)**
- Symptôme : SaaS « sain » vs nœud « aucun standby » → deux sources de vérité divergentes
- Cause : standby auto-promu pendant que le primaire tournait → deux primaires
- Solution ingénieur : réparation `pg_basebackup` + faire dire la vérité au SaaS (mesure `pg_stat_replication`)
- Principe : jamais de dégradation silencieuse — de l'incident à l'amélioration produit (agent IA)

*Message clé :* La valeur de l'ingénieur n'est pas la vitesse d'exécution mais la qualité du diagnostic et la pérennité de la solution.

---

### Partie 10 — Bilan, Compétences & Perspectives (2 min — Slides 15–16)

**Slide 15 — Compétences**
- Rust, libsodium/zero-knowledge, PostgreSQL/systèmes distribués, IA & Big Data, Docker, Django

**Slide 16 — Bilan & perspectives**
- Acquis : 6/6 scénarios, zero-knowledge prouvé, reporting véridique, **agent IA de supervision implémenté** (clusters + serveur éditeur, temps réel, rapports PDF)
- Perspectives : fencing (failover sûr), IP statiques, **modèle prédictif entraîné** (Isolation Forest / z-série) sur l'historique de métriques, module métier v1

*Message clé :* Un socle prouvé, avec une feuille de route technique claire.

---

### Partie 11 — Conclusion (1 min — Slide 17)

**Slide 17 — Message final**
- Un éditeur peut vendre en SaaS complet sans jamais pouvoir lire les données de ses clients
- 6/6 scénarios validés, conforme AUDPF par design
- Un projet technique + stratégique + humain, ancré dans ma spécialité IA & Big Data

Citation de clôture (optionnelle) :
> *"La souveraineté numérique n'est pas un luxe technologique — c'est une condition de l'autonomie stratégique des organisations africaines."*

---

## Questions Jury — Préparation

| Question probable | Réponse préparée |
|-------------------|-----------------|
| Comment garantissez-vous que l'éditeur ne peut PAS lire les données ? | Hiérarchie de clés : la DEK (unique par entreprise) ne quitte jamais le périmètre client en clair. Le relais ne stocke que des blobs chiffrés et une DEK scellée sous le code de récupération du client, que l'éditeur ne connaît pas. C'est cryptographique, pas contractuel. |
| Pourquoi PostgreSQL et pas un consensus maison (Raft/Paxos) ? | Réinventer un consensus serait une faute d'ingénierie : risque élevé, valeur nulle. On utilise la promotion de standby PostgreSQL + supervision type Patroni — éprouvé en production. |
| Qu'est-ce qui empêche le split-brain ? | À 2 nœuds, bascule manuelle uniquement. Le fencing par jeton d'époque monotone (prochaine étape) isole tout ancien primaire revenant avec une époque inférieure. J'ai rencontré le problème en vrai et corrigé le reporting pour le détecter. |
| En quoi votre projet relève-t-il de l'IA & Big Data ? | Le cluster produit des séries temporelles de métriques (heartbeats, état de réplication, bascules) stockées en base. J'ai **implémenté** un agent de supervision : il analyse en temps réel ces métriques et l'état du serveur éditeur via un LLM (Mistral), et rend un verdict (risque, score d'anomalie, diagnostic, recommandation) avec rapport PDF. Honnêtement, l'étape suivante est un modèle non supervisé entraîné (Isolation Forest / z-score glissant) sur l'historique accumulé, pour passer du diagnostic à la **prédiction** de panne. |
| Pourquoi un LLM (Mistral) et pas directement un modèle ML entraîné ? | Pragmatisme et délai : un modèle supervisé exigeait un historique labellisé de pannes que je n'avais pas. Le LLM raisonne immédiatement sur les métriques et produit une explication lisible par l'admin. L'agent est conçu pour basculer vers un modèle entraîné quand l'historique sera suffisant — l'infrastructure de collecte est déjà en place. |
| L'agent IA peut-il voir les données métier ? | Non. Il ne reçoit que des métriques d'infrastructure (CPU, RAM, réplication, heartbeats). Aucune donnée métier ne lui est transmise — la garantie zero-knowledge est préservée jusque dans la supervision. |
| L'agent supervise-t-il aussi le relais ? Ne risque-t-il pas de lire les sauvegardes ? | Oui, il supervise les 3 acteurs (éditeur, relais, clusters), mais pour le relais il interroge uniquement l'endpoint de santé `/health` qui ne renvoie que des métadonnées : disponibilité, uptime, *nombre* de tenants ayant des blobs. Le contenu des blobs est chiffré et inaccessible par conception — l'agent n'a aucun chemin pour le lire. La souveraineté est structurelle, pas déclarative. |
| Le zero-knowledge ralentit-il les performances ? | Le chiffrement se fait côté client (XChaCha20-Poly1305, très rapide). La réplication PostgreSQL n'est pas affectée : elle réplique le journal chiffré. Réplication confirmée en < 1 s sur le banc. |
| Quel est le modèle économique / prix ? | Licences par sièges, 3 plans dégressifs : Starter 149 DH, Pro 119 DH, Enterprise 89 DH par poste/mois. C'est 2 à 5 fois moins cher que les SaaS classiques (Odoo, Sage, Dynamics : 250–1 100 DH/poste/mois). Raison structurelle : l'éditeur n'héberge pas les données du client (elles restent sur ses machines), donc son coût marginal est faible — il peut baisser le prix par poste tout en gardant sa marge. Pour une PME de 10 postes : ~14 000 DH/an au lieu de ~48 000, soit 70 % d'économie, données souveraines en plus. Chiffres = estimations cohérentes à affiner avec AL BARAA. |
| Comment l'éditeur gagne-t-il de l'argent s'il ne vend pas d'hébergement ? | Il facture la valeur réelle : le logiciel métier, les mises à jour centralisées, le support, la gestion du parc et la sauvegarde chiffrée sur le relais. Il économise les coûts d'infrastructure cloud (compute + stockage des données clients), ce qui améliore sa marge à prix de vente plus bas. Modèle gagnant-gagnant. |
| Avez-vous eu des échecs lors des tests ? | 0 échec sur les 6 scénarios. Le scénario de panne (T6) a même validé deux choses : le rattrapage WAL et la détection de panne par le SaaS. |

---

## Conformité aux consignes EIGSI

**Les 5 axes d'évaluation du jury :**

| Axe | Où c'est couvert |
|-----|------------------|
| 1. Structure de l'exposé | Sommaire chronométré (~31 min), enchaînement contexte → problème → solution → résultats → conclusion |
| 2. Message technique | Justification des choix (PostgreSQL vs consensus maison, libsodium, zero-knowledge) — Parties 4-5, réserves B-D |
| 3. Réponses aux questions | Section « Questions Jury — Préparation » (6 Q/R argumentées) |
| 4. Visuels et supports | Slides avec schémas, captures démo réelle, tableaux de résultats — notes design par slide |
| 5. Expression orale | Notes présentateur par slide (ton, regard, phrases d'accroche) dans le support |

**Checklist contenu (consigne) :** Contexte ✅ (P1) · Problématique ✅ (P2) · Objectifs ✅ (P2, SMART) · Missions ✅ (P1) · Réalisations ✅ (P5-7) · Résultats mesurables ✅ (P7, 6/6) · Compétences ✅ (P10) · Plan logique ✅.

---

## Checklist Avant Dépôt (24/06/2026)

- [ ] Titre et thème à jour : "Framework SaaS Souverain" (plus de "SDA / Coffre-Fort P2P")
- [ ] Le contexte AL BARAA et la mission sont clairement définis (slide 3)
- [ ] La problématique (dilemme de l'éditeur) est formulée précisément (slide 4)
- [ ] Architecture 3 acteurs + zero-knowledge clairs (slides 5–9)
- [ ] Résultats quantifiés : 6/6 scénarios, réplication < 1 s (slide 11)
- [ ] Agent IA présenté comme implémenté et démontrable (slide 12) ; modèle prédictif entraîné = perspective honnête
- [ ] Incident split-brain présenté comme posture ingénieur (slide 14)
- [ ] Compétences explicitées, lien IA & Big Data (slide 15)
- [ ] Durée totale : 27–33 minutes chronométrée
- [ ] PDF produit et prêt pour dépôt Moodle

---

*Plan de Soutenance — MPIGA-ODOUMBA Jesse — EIGSI Promo 2026*
*Soutenance : 01/07/2026 à 10h00 — EIGSI Casablanca*
