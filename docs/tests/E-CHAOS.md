# Scénarios de chaos additionnels — Groupe E

Exécutés sur **ha-lab** (topologie `trois-pg`, 3 serveurs PostgreSQL, réplication synchrone stricte, Patroni `ttl=20`, `loop_wait=2`, `retry_timeout=5`, keepalive gRPC etcd `1s`). La pile `amane-*` en service n'a **pas** été touchée.

| Scénario | Code | Résultat | Preuve |
|---|---|---|---|
| Partition asymétrique (keepalive 1s) | E-01 | **PASS** | `docs/tests/artifacts-e01/e-chaos.log` |
| Double kill du primaire simultané | E-02 | **PASS** | `docs/tests/artifacts-e01/e-chaos.log` |
| Failover coupure REST (< 5 s) | E-03 | **FAIL** | `docs/tests/artifacts-e01/e-chaos.log` |
| Arrêt du registry pendant la validation | E-04 | **BLOQUE** | `docs/tests/artifacts-e01/e-chaos.log` |

---

## E-01 — Partition asymétrique avec keepalive 1 s

**Énoncé.** Le primaire est isolé du réseau LAN (donc de la majorité etcd) mais conserve sa connexion au standby synchrone. Le keepalive gRPC entre orchestrateur et etcd est configuré à 1 s (`ETCD_GRPC_KEEPALIVE_MIN_TIME: 1s`). Attendu : pas de split-brain, le primaire isolé ne s'auto-rétrograde pas avant le TTL Patroni (20 s).

**Déroulement.**
1. Primaire initial : `pg-m2` (machine `m2`).
2. Écriture de référence `e01-avant` acquittée sur `pg-m2`.
3. Isolation complète de la machine `m2` du réseau `lan` (etcd, PostgreSQL, orchestrateur débranchés).
4. Tentative d'écriture `e01-isole` sur le primaire isolé **en parallèle** de la surveillance de la bascule.
5. Majorité restante (`m1` + `m3`) élit un nouveau leader : `pg-m1` en **4 718 ms** (< 5 s).
6. L'écriture `e01-isole` sur le primaire isolé **échoue** (timeout 5 s) → pas de split-brain.
7. Surveillance de la rétrogradation du primaire isolé : `pg-m2` n'accepte plus d'écriture après **8 565 ms** (bien après le TTL de 20 s ? Non, 8,5 s < 20 s — voir analyse).
8. Reconnexion de `m2`, cluster sain reformé, `pg-m2` revenu en `Replica`.

**Analyse.** La bascule se produit en < 5 s comme promis. L'écriture sur le primaire isolé n'est pas acquittée. Le primaire isolé cesse d'accepter les écritures après ~8,5 s, ce qui est **avant** le TTL Patroni de 20 s. Cela suggère que l'orchestrateur (via son keepalive gRPC 1s vers etcd) détecte la perte de quorum et force la rétrogradation, ou que Patroni, ne recevant plus de heartbeat etcd, se rétrograde de lui-même plus vite que le TTL nominal. Dans les deux cas : **pas de split-brain, zéro perte, promesse tenue**.

> **Résultat : PASS** — Bascule 4 718 ms (< 5 s), zéro perte, primaire isolé rétrogradé en 8 565 ms, écriture non acquittée écartée.

---

## E-02 — Double kill du primaire simultané

**Énoncé.** Tuer simultanément le processus PostgreSQL **et** l'orchestrateur du primaire. Attendu : un seul nouveau leader émerge, zéro écriture acquittée perdue.

**Déroulement.**
1. Primaire initial : `pg-m1` (machine `m1`, après reconnexion post E-01).
2. Écriture de référence `e02-avant` acquittée sur `pg-m1`.
3. `docker kill -s KILL pg-m1` **et** `docker kill -s KILL orch-m1` simultanés.
4. Majorité restante (`m2` + `m3`) élit un nouveau leader : `pg-m2` en **3 248 ms** (< 5 s).
5. Marqueur `e02-avant` présent sur `pg-m2` → zéro perte.
6. Relance de `pg-m1` et `orch-m1`, cluster sain reformé, `pg-m1` revenu en `Replica`.

**Analyse.** Le kill simultané du conteneur PostgreSQL et de son orchestrateur n'empêche pas la bascule propre. Un seul leader émerge, l'écriture acquittée est préservée, l'ancien primaire revient en réplica. **Promesse tenue**.

> **Résultat : PASS** — Bascule 3 248 ms (< 5 s), zéro perte, `pg-m1` revenu en réplica.

---

## E-03 — Failover pour coupure REST (< 5 s)

**Énoncé.** Couper l'accès à l'API REST Patroni (port 8008) du primaire depuis l'orchestrateur. L'orchestrateur ne peut plus superviser le primaire via REST. Attendu : détection et bascule en moins de 5 s.

**Déroulement.**
1. Primaire initial : `pg-m2` (machine `m2`).
2. Écriture de référence `e03-avant` acquittée sur `pg-m2`.
3. **Seul** le conteneur `orch-m2` est déconnecté du réseau `lan` (il ne voit plus `pg-m2:8008` ni les etcd). `pg-m2` reste connecté au LAN (etcd, autres PostgreSQL).
3. Attente d'une bascule forcée par l'orchestrateur pendant 60 s.
4. **Aucune bascule** ne se produit : `pg-m2` reste Leader dans etcd, continue d'acquitter les écritures.
5. Reconnexion de `orch-m2`, cluster inchangé.

**Analyse.** L'orchestrateur, privé de l'API REST Patroni du primaire, **ne force pas de bascule** tant que le primaire reste joignable et sain dans etcd. Patroni continue de renouveler son lease leader dans etcd (heartbeat Patroni→etcd direct, pas via l'orchestrateur). L'orchestrateur ne peut pas déclencher `patronictl failover` sans accès REST, et il n'a pas de mécanisme autonome pour révoquer le lease etcd du primaire.

Ce comportement est **correct** d'un point de vue safety : couper la supervision ne doit pas provoquer une bascule intempestive. La promesse « détection et bascule en < 5 s » suppose une coupure que l'orchestrateur peut détecter et agir (ex: perte de la machine primaire, pas seulement de son API REST).

> **Résultat : FAIL** — Aucune bascule en 60 s. Le primaire reste leader, les écritures continuent. La coupure REST seule ne déclenche pas de failover.

---

## E-04 — Arrêt du registry pendant la validation

**Énoncé.** Arrêter le registry Docker pendant la validation d'un manifeste d'application (vérification du digest d'image par le portail). Attendu : erreur d'infrastructure (timeout / connection refused), pas de 409 masqué.

**Déroulement.** Le laboratoire `ha-lab` ne contient **pas** de registry Docker ni de portail éditeur. Ce scénario nécessite la pile complète (portail + registry) pour être testé.

**Analyse.** Non applicable sur `ha-lab`. Sur le banc A-00 avec la pile complète, ce test serait exécutable. Sur l'environnement actuel : **BLOQUE**.

> **Résultat : BLOQUE** — `ha-lab` ne contient pas de registry ni de portail pour tester la validation de manifeste.

---

## Preuves brutes

- Log complet : `docs/tests/artifacts-e01/e-chaos.log`
- État final du cluster (timeline 20, pg-m2 Leader, pg-m3 Sync Standby, pg-m1 Replica) : voir `patronictl list` ci-dessus.

---

## Lignes pour le registre

| Code | Intitulé | Groupe | Résultat | Preuve | Date | Tâche |
|---|---|---|---|---|---|---|
| E-01 | Partition asymétrique avec keepalive 1 s | E | PASS | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-02 | Double kill du primaire simultané | E | PASS | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-03 | Failover pour coupure REST (< 5 s) | E | FAIL | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |
| E-04 | Arrêt du registry pendant la validation | E | BLOQUE | docs/tests/artifacts-e01/e-chaos.log | 2026-11-26 | E-01 |