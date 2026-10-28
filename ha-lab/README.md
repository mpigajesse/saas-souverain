# Laboratoire HA — trois machines simulées

Banc de preuve de la haute disponibilité PME (refonte V3, sous-étape 2a,
ADR-001 et ADR-004 du journal `docs/refonte-v3/AMANE-Refonte-V3.md`).

Sur un seul poste, chaque « machine » `mN` est un groupe de conteneurs relié au
réseau `lan` (le réseau de la PME) et à un réseau privé `mN` (son bus local) :

| Machine | Topologie `temoin` (défaut) | Topologie `trois-pg` |
|---|---|---|
| m1 | etcd + PostgreSQL/Patroni + orchestrateur | idem |
| m2 | etcd + PostgreSQL/Patroni + orchestrateur | idem |
| m3 | **témoin etcd seul** | etcd + PostgreSQL/Patroni + orchestrateur |
| Synchrone | non strict (alerte à lever, étape 2d) | strict |

## Lancer les scénarios de chaos

Prérequis : Docker démarré, image `ghcr.io/zalando/spilo-17:4.0-p3` (tirée au
premier lancement), image de l'orchestrateur construite par compose.

```bash
bash ha-lab/chaos.sh            # les deux topologies
bash ha-lab/chaos.sh temoin     # 2 PostgreSQL + témoin etcd
bash ha-lab/chaos.sh trois-pg   # 3 PostgreSQL, synchrone strict
```

Chaque topologie est recréée à neuf (`docker compose down -v`). Le script
termine par un bilan et rend le nombre de scénarios en échec.

| Scénario | Promesse vérifiée |
|---|---|
| Crash du PostgreSQL primaire | Bascule automatique, zéro perte, retour de l'ancien primaire en réplica |
| Panne de la machine du primaire | La majorité etcd restante (témoin compris) suffit à basculer |
| Isolement réseau du primaire | Le primaire isolé n'acquitte plus aucune écriture et se rétrograde seul ; une écriture non acquittée n'apparaît jamais dans le cluster |
| Perte du standby (non strict) | Les écritures reprennent, le cluster passe en asynchrone |
| Isolement du témoin etcd | Rien ne change |
| Partition minoritaire d'un standby | Aucune promotion côté minorité |
| Blocage strict sans standby | Aucune écriture acquittée sans standby synchrone |

## Adressage

Sur `lan`, chaque conteneur a une IP fixe `10.77.0.<machine><rôle>` (rôle 1 =
etcd, 2 = PostgreSQL/Patroni, 3 = orchestrateur) et les services se désignent
par IP, comme les machines d'une PME. Un nom de conteneur arrêté met plusieurs
secondes à échouer en DNS Docker, ce qui fausserait les mesures.

## Isoler une machine à la main

```bash
for c in etcd-m1 pg-m1 orch-m1; do docker network disconnect amane-ha-lab_lan $c; done
docker exec pg-m2 patronictl list
docker network connect --ip 10.77.0.11 amane-ha-lab_lan etcd-m1
docker network connect --ip 10.77.0.12 amane-ha-lab_lan pg-m1
docker network connect --ip 10.77.0.13 amane-ha-lab_lan orch-m1
```

## Comportements à connaître

- Après une bascule forcée, l'orchestrateur s'interdit d'en forcer une autre
  pendant 30 s (anti-oscillation) : une seconde panne dans cette fenêtre n'est
  rattrapée que par le ttl de Patroni (~20 s). Le script attend donc la fin de
  ce délai entre deux scénarios de bascule.
- Un primaire privé d'etcd est arrêté par Patroni (« demoting self, offline ») ;
  son API REST peut encore renvoyer le dernier rôle connu. Le critère retenu est
  donc que PostgreSQL n'accepte plus d'écriture.

Arrêt complet : `docker compose -f ha-lab/docker-compose.yml --profile trois-pg down -v`.
