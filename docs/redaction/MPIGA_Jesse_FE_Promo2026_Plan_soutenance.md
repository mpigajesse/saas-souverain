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
| 3 | La solution : un framework à trois acteurs | 3 min | 5–6 |
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

---

### Partie 3 — La Solution : Trois Acteurs (3 min — Slides 5–6)

**Slide 5 — Architecture à trois acteurs**
- SaaS éditeur (Django) : comptes tenants, licences, suivi du parc — voit compte/licence seulement
- Relais zero-knowledge : stocke des blobs chiffrés opaques — ne voit **rien**
- Cluster PME : exécute le logiciel, détient les données **en clair** dans son périmètre

**Slide 6 — Positionnement**
- vs SaaS cloud classique (expose les données) · vs logiciel installé (perd le SaaS)
- 3 différenciateurs : SaaS complet sans accès données / résilience BDD / souveraineté cryptographique

*Message clé :* On découpe le rôle en trois pour séparer ce qui peut être vu de ce qui ne doit jamais l'être.

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
5. Coupure du standby → SaaS détecte « ✗ Réplication interrompue »
6. Redémarrage → rattrapage WAL automatique + retour « sain »

*Message clé :* Deux machines réelles, réplication transparente, et un SaaS qui ne ment jamais sur l'état réel.

---

### Partie 7 — Résultats et Validation (2 min — Slide 11)

**Slide 11 — 6/6 scénarios validés**
- Réplication, base partagée 2 employés, cohérence ACID + alerte seuil
- Unicité (contrainte UNIQUE), concurrence MVCC, panne/rattrapage WAL
- 0 échec, réplication streaming confirmée par `pg_stat_replication`

*Message clé :* Pas une démo de laboratoire — une campagne de test structurée sur banc multi-OS.

---

### Partie 8 — Supervision Intelligente : Agent IA (3 min — Slide 12)

**Slide 12 — L'angle Big Data & IA**
- Le cluster émet des séries temporelles : heartbeats (~60 s), streaming_standby_count, WAL lag, failover_count
- Agent IA = détection d'anomalie non supervisée (Isolation Forest / z-score) sur ces métriques
- Apport vs seuil fixe : **prédiction** de panne avant franchissement, pas réaction après
- Preuve de concept : détection de l'anomalie ayant précédé le split-brain observé en test

*Message clé :* Ma spécialité IA & Big Data s'applique directement à un problème d'infrastructure réel : superviser intelligemment le cluster.

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
- Acquis : 6/6 scénarios, zero-knowledge prouvé, reporting véridique, agent IA (PoC)
- Perspectives : fencing (failover sûr), IP statiques, agent IA en production, module métier v1

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
| En quoi votre projet relève-t-il de l'IA & Big Data ? | Le cluster produit des séries temporelles de métriques. J'ai conçu un agent de détection d'anomalies (non supervisé) qui apprend le comportement normal et prédit les pannes — un cas d'usage Big Data + IA ancré dans des données réelles. |
| Le zero-knowledge ralentit-il les performances ? | Le chiffrement se fait côté client (XChaCha20-Poly1305, très rapide). La réplication PostgreSQL n'est pas affectée : elle réplique le journal chiffré. Réplication confirmée en < 1 s sur le banc. |
| Avez-vous eu des échecs lors des tests ? | 0 échec sur les 6 scénarios. Le scénario de panne (T6) a même validé deux choses : le rattrapage WAL et la détection de panne par le SaaS. |

---

## Checklist Avant Dépôt (24/06/2026)

- [ ] Titre et thème à jour : "Framework SaaS Souverain" (plus de "SDA / Coffre-Fort P2P")
- [ ] Le contexte AL BARAA et la mission sont clairement définis (slide 3)
- [ ] La problématique (dilemme de l'éditeur) est formulée précisément (slide 4)
- [ ] Architecture 3 acteurs + zero-knowledge clairs (slides 5–9)
- [ ] Résultats quantifiés : 6/6 scénarios, réplication < 1 s (slide 11)
- [ ] Agent IA présenté avec preuve de concept (slide 12)
- [ ] Incident split-brain présenté comme posture ingénieur (slide 14)
- [ ] Compétences explicitées, lien IA & Big Data (slide 15)
- [ ] Durée totale : 27–33 minutes chronométrée
- [ ] PDF produit et prêt pour dépôt Moodle

---

*Plan de Soutenance — MPIGA-ODOUMBA Jesse — EIGSI Promo 2026*
*Soutenance : 01/07/2026 à 10h00 — EIGSI Casablanca*
