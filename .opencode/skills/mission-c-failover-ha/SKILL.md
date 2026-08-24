---
name: mission-c-failover-ha
description: Use when proving the Mission C failover < 5s target and HA: switchover measurement protocol, crash vs network partition, lease fencing check, zero-data-loss verification, toxiproxy.
---

# Skill : Failover & Tests HA — prouver le < 5 s
**Jalon concerné :** 2 à 4 — mesure le consensus (02) et la réplication (04) dans les conditions réelles
**Pourquoi :** un des livrables chiffrés de Mission C est le **failover < 3-5 s**. Un failover non mesuré n'existe pas — il faut un protocole de test reproductible qui le démontre, pas des anecdotes.

## Concepts clés

- **Fenêtre de bascule** : `temps de détection (lease TTL) + temps d'élection (Raft etccc) + temps de promotion Patroni`. Séparer ces trois étages permet de savoir OÙ le budget de 5s part.
- **Split-brain** : deux primaires concurrents pendant la bascule. Le **lease fencing** etcd est la garantie anti-split-brain — le test doit vérifier que l'ancien primary est bien fencé (il ne ré-accepte pas les écritures en revenant).
- **Quorum** : avec N nœuds, la majorité est ceil(N/2)+1. Un cluster de 3 tolère 1 nœud mort. Ne tester jamais un failover sur un cluster à 1 seul nœud — c'est du théâtre.
- **Deux types de panne ≠** : *process crash* (kill) et *partition réseau* (le primary tourne mais est isolé) exercent des chemins différents — il faut tester les deux.

## Protocole de test reproductible

```bash
docker compose ps > /tmp/ha-baseline
date +%s > /tmp/t0

docker stop postgres-replica

until curl -s localhost:8008/patroni | grep -q '"role":"primary"'; do sleep 0.2; done
echo "failover: $(( $(date +%s) - $(cat /tmp/t0) )) s"

docker start postgres-replica
curl -s localhost:8008/patroni | grep -o '"role":"replica"'
```

## Variante important : panne réseau, pas crash

```bash
iptables -A INPUT  -s <ip replica> -p tcp --dport 2379,5432 -j DROP
iptables -A OUTPUT -d <ip replica> -p tcp --dport 2379,5432 -j DROP
iptables -D INPUT  -s <ip replica> -p tcp --dport 2379,5432 -j DROP
iptables -D OUTPUT -d <ip replica> -p tcp --dport 2379,5432 -j DROP
```

Outils plus propres si dispo : **Toxiproxy** ou **testcontainers-go** avec des *toxics de réseau* (latency, drop, blackhole) pour automatiser ça en test.

## Métriques à enregistrer à chaque run

- `detection_ms` : du crash à la révocation du lease dans etcd.
- `election_ms` : du lease libre à la prise du lock par le nouveau primary.
- `promotion_ms` : du lock pris à `role=primary` + port acceptant des connexions.
- `zero_data_loss` : le nouveau primary doit être **strictement à jour** (réplication synchrone) — vérifier qu'aucune transaction ackée avant le crash n'est perdue.

## Cas du superviseur maison (option C — crash < 5 s)

Depuis l'intégration du superviseur (package `orchestrator-go/supervisor/`), le crash non contrôlé
n'est plus borné par la lease Patroni (`ttl >= 20 s`) : le superviseur publie un **heartbeat propre**
dans etcd (lease 2 s) et sonde le primary (REST, 500 ms) ; quand le nœud est confirmé mort
(heartbeat stale/absent **et** probe en échec, StaleConfirm=2 ticks), il **libère le lock**
`/service/<scope>/leader` dans etcd puis appelle `POST /failover` **sans le champ `leader`**.
Mesuré sur la vraie stack (superviseur actif) : **crash** failover ~3,2 s, writable ~4,0 s —
cible < 5 s **atteinte** (avant : ~21 s) ; **partition** (coupure complète, `docker network
disconnect` du primary) : failover ~3,0 s, writable ~3,4 s, détection ~0,7 s — cible < 5 s
**atteinte** aussi (avant : ~21 s). Garde anti-partition : heartbeat frais (primary vivant mais
isolé en split-brain partiel) ⇒ **jamais de forçage**, Patroni tranche ; en **coupure complète**
le heartbeat du primary stoppe (injoignable pour tous) ⇒ forçage légitime — c'est le cas mesuré
à 3,0 s. **Agent heartbeat local** : en déploiement 3 nœuds, chaque superviseur ne publie/supprime
le heartbeat QUE de son nœud local (`Config.LocalNode` ← `AMANE_NODE_ID`) — sinon il supprimerait le
heartbeat du primary quand SON lien REST tombe et la garde anti-partition serait inopérante.
`LocalNode` vide (défaut) = mode mono-hôte. Prouvé vs etcd réel :
`AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run 'PartitionGuardAgainstEtcd' -v`
(REST down + heartbeat frais ⇒ JAMAIS de forçage, lock Patroni intact — ce cas a révélé le bug du
suppresseur non-local) et `SupervisorAgainstEtcd` (#crash# ⇒ `/failover` < 5 s).
Le protocole de mesure ci-dessus reste valable tel quel
(`failover_measure.sh crash|partition` avec superviseur actif) ; le fencing (nœud confirmé mort qui ne
recapture pas le lock) et le zéro perte se vérifient de la même façon.

## Pièges courants

- Mesurer avec un `sleep 5` plutôt qu'un timing réel → ne mesure rien. Il faut un horodatage précis (µs) déclenché au crash.
- Tester le failover sur un seul nœud → le "failover" est trivial et ne dit rien sur le quorum.
- Oublier de vérifier le **fencing** (ancien primary qui ne reprend pas les écritures) — le test "passé" alors que le cluster est en split-brain masqué.
- Ne jamais redémonter le sevice, ne pas remettre le cluster dans son état d'origine après le test.

## Vérifier la version

- Les API Patroni (`/switchover`, `/restart`) et les clients etcd (`client/v3`) bougent régulièrement — vérifier les chemins et formats au moment du test. Toxiproxy et testcontainers-go ont des cycles de release fréquents : activer/pinner leurs versions en CI.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Patroni replication modes (synchronous): `patroni.readthedocs.io/en/latest/replication_modes.html`
- Toxiproxy : `github.com/Shopify/toxiproxy` ; testcontainers-go : `golang.testcontainers.org`