# Registre des résultats de tests — AMANE

Ce fichier est **la** source unique des résultats. Chaque tâche du pilote doit
y ajouter sa ligne avant de rendre la main. Un test qui n'est pas ici n'est pas
compté, même s'il a été exécuté.

Il est volontairement plat et analysable : `tools/registre.py` le relit et
produit la synthèse de couverture. Ne pas transformer ce tableau en prose.

## Convention

- **Code** : identifiant du plan de référence (`C-01`, `N-03`, `L-04`, `D-01`,
  `HA-02`, `R-01`, `E2E-01`) ou `A-xx` pour une épreuve de campagne.
- **Résultat** : `PASS`, `FAIL`, `BLOQUE`, `NON TENU`, `SKIP`.
  - `PASS` : l'assertion du plan est vérifiée par une preuve.
  - `FAIL` : l'assertion est vérifiée et elle est fausse.
  - `BLOQUE` : non exécutable en l'état (dépendance absente, environnement).
  - `NON TENU` : exécuté, cible de performance ou de charge non atteinte.
  - `SKIP` : non pertinent sur ce banc, avec la raison.
- **Preuve** : chemin vers la sortie brute ou le rapport, jamais « cf. message ».
- Toute ligne doit être **reproductible** : la commande exacte figure dans le
  rapport cité, pas dans ce registre.

## Synthèse de couverture

| Groupe | Plan | Prévus | Passés | Échecs | Bloqués | Non tenus | Ignorés |
|---|---|---:|---:|---:|---:|---:|---:|
| C — socle cryptographique | C | 6 | 3 | 2 | 1 | 0 | 0 |
| N — nœud PME | N | 6 | 3 | 3 | 0 | 0 | 0 |
| R — relais zero-knowledge | R | 3 | 2 | 1 | 0 | 0 | 0 |
| D — portail éditeur | D | 6 | 1 | 5 | 0 | 0 | 0 |
| HA — haute disponibilité | HA | 6 | 6 | 0 | 0 | 0 | 0 |
| E — chaos | E | 4 | 2 | 1 | 1 | 0 | 0 |
| M — acceptation E2E | M | 3 | 3 | 0 | 0 | 0 | 0 |
| L — charge | L | 12 | 9 | 0 | 0 | 2 | 0 |
| A — campagne | A | 10 | 9 | 0 | 0 | 1 | 0 |

## Registre

| Code | Intitulé | Groupe | Résultat | Preuve | Date | Tâche |
|---|---|---|---|---|---|---|
| C-01 | Corruption part Shamir interceptée par AEAD | C | PASS | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| C-02 | Vecteurs RFC 7748 de petit ordre et point à l'infini (seal_dek / open_sealed_dek) | C | FAIL | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| C-03 | Troncatures du sealed box (0/31/32/79 octets) et altération de la clé éphémère | C | PASS | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| C-04 | Zéroïsation mémoire (ZeroizeOnDrop) inspectée en unsafe après drop | C | PASS | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| C-05 | Torn write (len=256) et en-tête DoS (len=0xFFFFFFFF) dans ss-journal | C | FAIL | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| C-06 | Régression d'époque (epoch=4 après epoch=5, JournalError::EpochRegression) | C | BLOQUE | docs/tests/artifacts-c01/ | 2026-11-25 | C-01 |
| N-01 | Split-brain dans la garde de quorum (deux Role=Leader) | N | FAIL | docs/tests/artifacts-n01/harness-n01-n05.log | 2026-11-25 | N-01 |
| N-02 | Fencing continu par timeline PostgreSQL (locale 2, cluster 3) | N | PASS | docs/tests/artifacts-n01/harness-n01-n05.log | 2026-11-25 | N-01 |
| N-03 | DoS codes d'enrôlement (5 requêtes invalides, rate-limit, temps constant) | N | FAIL | docs/tests/artifacts-n01/node-tests-nocapture.log | 2026-11-25 | N-01 |
| N-04 | Streaming mémoire backup pg_dump 5 Go (RSS<128 Mo, AEAD par morceau) | N | FAIL | docs/tests/N-NOEUD.md | 2026-11-25 | N-01 |
| N-05 | Atomicité restore (blob altéré 1 bit, échec avant pg_restore) | N | PASS | docs/tests/artifacts-n01/harness-n01-n05.log | 2026-11-25 | N-01 |
| N-06 | Étanchéité reverse proxy (hop-by-hop, normalisation, isolation, 502) | N | PASS | docs/tests/artifacts-n01/harness-n06.log | 2026-11-25 | N-01 |
| R-01 | Isolation multi-tenant : jeton A sur blob de B (GET/PUT/DELETE), HMAC du tenant_id | R | FAIL | docs/tests/artifacts-r01/r01-sortie.txt | 2026-11-25 | R-01 |
| R-02 | Coupure TCP à 20 % d'un blob de 500 Mo : aucun résidu, GET=404, ré-émission | R | PASS | docs/tests/artifacts-r01/r02-preuves-complementaires.md | 2026-11-25 | R-01 |
| R-03 | Liaison cryptographique : blob amane copié en application refusé (AAD tenant/base/date) | R | PASS | docs/tests/artifacts-r01/r03-sortie.txt | 2026-11-25 | R-01 |
| D-01 | Jeton valide du tenant B + installation_id du tenant A (usurpation) | D | FAIL | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| D-02 | Enregistrement sur licence expirée (403 license_expired, is_active=False prioritaire) | D | FAIL | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| D-03 | Deux POST register concurrents, licence à 1 siège (1×201, 1×403, actif=1) | D | FAIL | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| D-04 | Confinement du compose généré (réseau internal, app sans port, PG/etcd ailleurs) | D | PASS | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| D-05 | Manifeste dangereux (docker.sock, /etc, /proc, sh -c curl, file://, gopher://) | D | FAIL | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| D-06 | Registre 401/403/500 (infrastructure, pas 409) et SSRF 302 vers 169.254.169.254 | D | FAIL | docs/tests/artifacts-d01/ | 2026-11-25 | D-01 |
| HA-01 | Élection de lease etcd : un seul leader, followers, aucune double promotion | HA | PASS | docs/tests/artifacts-ha01/ha01-election.log | 2026-11-25 | HA-01 |
| HA-02 | Mutex « force-right » : un opérateur ne peut pas forcer deux bascules | HA | PASS | docs/tests/artifacts-ha01/ha02-06-harness.log | 2026-11-25 | HA-01 |
| HA-03 | Keepalive gRPC : session survit à une coupure courte, reprise sans double enrôlement | HA | PASS | docs/tests/artifacts-ha01/ha02-06-harness.log | 2026-11-25 | HA-01 |
| HA-04 | Bascule HAProxy : backend suit le leader en < 5 s, aucun routage non-leader | HA | PASS | docs/tests/artifacts-ha01/ha04-bascule.log | 2026-11-25 | HA-01 |
| HA-05 | Hystérésis de synchronisation : pas d'oscillation du mode synchrone au seuil | HA | PASS | docs/tests/artifacts-ha01/ha02-06-harness.log | 2026-11-25 | HA-01 |
| HA-06 | Bootstrap concurrent : deux nœuds démarrés ensemble → un seul cluster | HA | PASS | docs/tests/artifacts-ha01/ha02-06-harness.log | 2026-11-25 | HA-01 |

<!-- Les lignes sont ajoutées par les tâches, une par test, dans l'ordre de code. -->

| M-01 | E2E-01 : Créateur → mon-espace → installer serveur → attente premier démarrage | M | PASS | docs/tests/M-E2E.md | 2026-11-26 | M-01 |
| M-02 | E2E-02 : Login → Applications → brouillon JSON → Valider → édition désactivée → rattachement licence | M | PASS | docs/tests/M-E2E.md | 2026-11-26 | M-01 |
| M-03 | E2E-03 : /mon-espace 1 serveur → 2ᵉ apparaît → bannière témoin → installeur → enregistré → bannière disparaît | M | PASS | docs/tests/M-E2E.md | 2026-11-26 | M-01 |

| A-00 | Banc de trois machines virtuelles réelles (KVM, sans root) | A | PASS | docs/tests/BANC-VM.md | 2026-11-25 | A-00 |
| A-01 | Baseline des tests existants (Go, Rust, Django) | A | PASS | docs/tests/BASELINE.md | 2026-11-25 | A-01 |
| A-02 | Charge API Portail L-01 avec k6 | A | NON TENU | docs/tests/L01.md | 2026-11-25 | A-02 |
| A-03 | Inventaire réel du banc et de la pile | A | PASS | docs/tests/INVENTAIRE.md | 2026-11-25 | A-03 |
| A-04 | AMANE à vide, mesures 10 minutes | A | PASS | docs/tests/A-VIDE.md | 2026-11-25 | A-04 |
| A-05 | Audit sécurité de la pile en service | A | PASS | docs/tests/SECURITE.md | 2026-11-25 | A-05 |
| A-06 | Trois SaaS avec processus métier réel | A | PASS | docs/tests/APPS.md | 2026-11-25 | A-06 |
| A-07 | Réplication PostgreSQL et réplicabilité | A | PASS | docs/tests/REPLICATION.md | 2026-11-25 | A-07 |
| A-08 | Extinction de machine, quorum et bascule | A | PASS | docs/tests/QUORUM.md | 2026-11-25 | A-08 |
| A-09 | Souveraineté, protection du logiciel, installation | A | PASS | docs/tests/SOUVERAINETE.md | 2026-11-25 | A-09 |
| L-02 | Charge relais zero-knowledge (débit, latence) | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-03 | Charge nœud Rust (ss-node) | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-04 | Réplication synchrone sous charge | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-05 | Bascule sous charge (failover, non-perte) | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-06 | Sauvegarde chiffrée gros volume | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-07 | Restauration gros volume | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-08 | Isolation charge multi-tenant relais | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-09 | Portail multi-tenant (isolation non vérifiable) | L | NON TENU | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-10 | Latence réseau inter-machines sous charge | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-11 | HAProxy sous charge | L | PASS | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| L-12 | Performance disque chiffré (chiffrement absent) | L | NON TENU | docs/tests/L-02-12.md | 2026-11-26 | L-02 |
| E-01 | Partition asymétrique avec keepalive 1 s | E | PASS | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-02 | Double kill du primaire simultané | E | PASS | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-03 | Failover pour coupure REST (< 5 s) | E | FAIL | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-04 | Arrêt du registry pendant la validation | E | BLOQUE | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |

## Campagnes

| Campagne | Périmètre | Tâches | P0 couverts | Date | Synthèse |
|---|---|---|---|---:|---|
| 1 | État du système, sécurité, SaaS, souveraineté, charge, réplication, chaos | 10 | 0 / 6 | 2026-11-25 | `docs/tests/` |
| 2 | Tests fonctionnels P0 du plan exhaustif (C, N, R, D, HA) | 0 | 0 / 6 | à venir | — |

## Ce que ce registre ne dit pas encore

La campagne 1 couvre **l'état du système**, pas les **invariants du plan**. Les
groupes C, N, R, D, HA, E, M et L restent à zéro : ce n'est pas un oubli de
saisie, c'est du travail non fait. Le premier groupe qui compte pour la mise en
production est **C** (intégrité cryptographique), puis **N** (fencing, quorum),
puis **HA**.

## Historique des campagnes

| Campagne | Périmètre | Tâches | Date | Rapport de synthèse |
|---|---|---|---|---|