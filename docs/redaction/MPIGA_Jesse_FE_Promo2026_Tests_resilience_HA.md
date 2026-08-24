# Campagne de tests — Haute disponibilité & résilience du cluster PME

> Document annexe au rapport de stage — preuves de validation sur banc réel.
> Auteur : Jesse MPIGA-ODOUMBA · AL BARAA CONSULTING · Promotion 2026
> Banc : 2 VMs physiques sur LAN `192.168.200.0/24` — Kali Linux (primaire) + Ubuntu 26.04 (standby).
> Stack : PostgreSQL 16 en conteneur Docker, réplication streaming, nœud métier `ss-node` (Rust).

---

## 1. Objectif

Prouver, sur deux machines réelles, que le cluster PME résiste aux pannes **sans perte de données ni split-brain**, et qu'il se répare **automatiquement**. Quatre protections ont été conçues, implémentées puis validées :

1. **Slot de réplication** — le primaire ne recycle jamais le WAL nécessaire au standby.
2. **Garde-fou anti auto-promotion** — à 2 nœuds, jamais de bascule automatique (évite le split-brain).
3. **Fencing par timeline PostgreSQL** — un ancien primaire déchu se clôture lui-même.
4. **Déploiement autonome** — tout re-clone recrée le slot sans intervention manuelle.

---

## 2. Test 1 — Réplication par slot (correctif du recyclage WAL)

**Problème d'origine.** Sans slot de réplication et avec `wal_keep_size=64 Mo`, quand le primaire démarrait seul (avant le standby), il recyclait les segments WAL dont le standby avait besoin → réplication cassée au redémarrage décalé.

**Correctif.** Slot physique `standby_1` créé sur le primaire (idempotent) + `pg_basebackup --slot`, `wal_keep_size` porté à 512 Mo, `max_slot_wal_keep_size=4096 Mo` (borne la rétention).

**Preuve.**
```sql
SELECT slot_name, active, wal_status FROM pg_replication_slots;
 slot_name | active | wal_status
-----------+--------+------------
 standby_1 | t      | reserved
```
```sql
SELECT client_addr, state FROM pg_stat_replication;
   client_addr   |   state
-----------------+-----------
 192.168.200.130 | streaming
```

**Verdict : ✅** Le slot `active=t / reserved` garantit que le primaire conserve le WAL pour le standby. Le scénario « éteindre, rallumer le primaire seul puis le standby » ne casse plus la réplication, par construction.

---

## 3. Test 2 — Garde-fou anti auto-promotion (2 nœuds)

**Règle (décision actée).** 2 nœuds → bascule **manuelle** uniquement ; ≥ 3 nœuds → failover automatique par quorum. L'auto-promotion à 2 nœuds provoquerait un split-brain.

**Scénario.** Le primaire (Kali) est éteint pendant que le standby (Ubuntu) tourne.

**Preuve (logs `ss-node` du standby).**
```
[tick 86] ALERTE : WAL receiver inactif (3/3) — primaire injoignable ?
[tick 86] Primaire injoignable — cluster 2 nœud(s) (<3) : BASCULE MANUELLE requise (anti split-brain, décision n°2).
```

**Verdict : ✅** Le standby a **refusé de s'auto-promouvoir** à 2 nœuds. Le nœud interroge le SaaS pour connaître la taille réelle du cluster ; en dessous de 3 nœuds, ou si le SaaS est injoignable (fail-safe), aucune promotion automatique.

---

## 4. Test 3 — Fencing par timeline PostgreSQL (clôture du primaire déchu)

**Principe.** L'époque du cluster = le **timeline ID** de PostgreSQL, incrémenté nativement à chaque `pg_promote()` (aucun consensus écrit à la main). Un nœud qui se croit primaire avec un timeline **inférieur** à celui du cluster est un ancien primaire déchu : il se clôture (ne lance pas son serveur web → aucune écriture métier).

**Scénario.**
1. Cluster sain : Kali primaire (timeline 1), Ubuntu standby (timeline 1).
2. Promotion manuelle du standby : `SELECT pg_promote();` → Ubuntu devient primaire **timeline 2**, époque 2.
3. Kali rallumé en se croyant primaire (timeline 1) face au cluster époque 2.

**Preuve (logs `ss-node` de Kali au redémarrage).**
```
Époque   : timeline PostgreSQL = 1
⛔ NŒUD CLÔTURÉ (FENCED)
Époque de ce nœud (timeline PG) : 1
Époque courante du cluster       : 2
Un primaire plus récent existe. Ce nœud est un ancien
primaire déchu : il REFUSE de servir pour éviter le split-brain.
[fenced] En attente de re-clone manuel (époque 1 < 2).
```

**Défense en profondeur (2 couches).**
- **Côté SaaS** : un nœud qui se déclare primaire avec une époque périmée reçoit `409 fenced` et ne peut pas rétrograder le primaire légitime.
- **Côté nœud** : au démarrage et à chaud (toutes les 60 s), auto-clôture si `époque locale < époque cluster`.

**Verdict : ✅** Le split-brain est neutralisé au niveau applicatif : l'ancien primaire ne sert plus. Validé end-to-end sur le banc.

---

## 5. Test 4 — Déploiement autonome (slot recréé au re-clone)

**Objectif.** Prouver qu'un re-clone d'un standby recrée le slot **automatiquement**, sans aucune commande SQL manuelle.

**Scénario.** Effacement du `pg-data` du standby puis redémarrage avec le nouvel entrypoint.

**Preuve (logs `pg-node` du standby).**
```
[standby] Création du slot de réplication standby_1 (si absent)...
[standby] pg_basebackup terminé — réplication configurée (slot standby_1)
LOG:  entering standby mode
LOG:  started streaming WAL from primary at 0/17000000 on timeline 1
LOG:  database system is ready to accept read-only connections
```

**Verdict : ✅** Le slot s'est créé tout seul. Tout re-clone futur se répare sans intervention.

---

## 6. Restauration de l'état nominal

Après le test de fencing, le cluster a été ramené à son état normal (Kali primaire, Ubuntu standby) par re-clone du standby sur la lignée du primaire — confirmant au passage la procédure opérationnelle de remise en service.

```sql
SELECT pg_is_in_recovery();           -- f  → Kali primaire
SELECT client_addr, state FROM pg_stat_replication;  -- 192.168.200.130 | streaming
SELECT slot_name, active FROM pg_replication_slots;  -- standby_1 | t
```

---

## 7. Synthèse

| # | Protection | Garantie | Verdict |
|---|-----------|----------|:------:|
| 1 | Slot de réplication | Plus de recyclage WAL → réplication robuste au redémarrage décalé | ✅ |
| 2 | Anti auto-promotion (< 3 nœuds) | Pas de bascule automatique → pas de split-brain à 2 nœuds | ✅ |
| 3 | Fencing par timeline | Un ancien primaire déchu se clôture (deux couches) | ✅ |
| 4 | Déploiement autonome | Re-clone = slot recréé automatiquement | ✅ |

**Résultat global : 4 / 4 protections validées sur banc réel — 0 échec.**

> Ces tests complètent la campagne métier (6/6 scénarios : réplication, base partagée, ACID, unicité, MVCC, panne/rattrapage WAL). Ensemble, ils prouvent que le socle de résilience est solide **avant** toute logique métier — conformément à la règle d'or du projet (« le socle d'abord, le métier ensuite »).

---

## 8. Principe d'ingénierie sous-jacent

Aucune brique critique n'a été réinventée : la réplication, la promotion et le timeline viennent de **PostgreSQL**. Le travail a consisté à **orchestrer** ces mécanismes éprouvés et à les rendre **véridiques** (le SaaS ne reflète que ce qui est mesuré) et **autonomes** (auto-réparation, auto-clôture). C'est la traduction concrète de la posture ingénieur : préférer l'assemblage rigoureux de briques sûres à du code maison risqué.
