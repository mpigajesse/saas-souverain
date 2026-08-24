---
name: mission-c-etcd-patroni
description: Use when working on Mission C consensus (consensus/): etcd Raft, lease/fencing, leader election, Patroni failover, WAL control (max_slot_wal_keep_size, synchronous_commit), go.etcd.io/etcd client v3.
---

# Skill : etcd + Patroni
**Jalon concerné :** 2 — Intégration etcd + Patroni (`consensus/`)
**Pourquoi :** c'est le remplacement direct du supervisor SQLx fait maison de la V1. Vous n'écrivez pas Patroni, vous l'intégrez et l'orchestrez.

## Concepts clés

- **Raft** : algorithme de consensus. Avec 3 nœuds, majorité = 2/3 → tolère la perte d'un nœud sans interruption.
- **Lease** : bail à durée limitée (TTL ~30s dans Amane). Le primary le renouvelle en continu ; s'il s'arrête, le lease expire et un autre nœud peut le récupérer.
- **Watch** : mécanisme etcd pour être notifié en temps réel d'un changement de clé — c'est comme ça que le standby détecte qu'un lease s'est libéré, sans polling actif.
- **Lease fencing** : garantit qu'un ancien primary revenu après coupure ne peut plus écrire (son lease a expiré entre-temps).
- **Write gated (fencing applicatif)** : `grpcserver.Server.Write` est refusé tant que le nœud ne détient pas la lease etcd (`grpcserver.Leadership`, wire en prod `consensus.NewLeadership`) → `codes.FailedPrecondition` « write refusé : nœud non leader (fencing lease) » avant tout accès au journal. Test d'intégration : `AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run TestLeadershipFencing -v` (élection → jamais deux leaders → le fencing transfère le leadership au successeur).
- **Patroni ne parle jamais directement à un autre Patroni** — toute la coordination passe par etcd (`/leader`, `/members`, `/config`).

## Commandes essentielles

```bash
etcdctl endpoint status --cluster -w table
etcdctl get /leader
etcdctl watch /leader

curl http://localhost:8008/           # état du nœud (primary/replica/running)
curl http://localhost:8008/cluster    # vue du cluster complet
curl -X POST http://localhost:8008/switchover  # bascule manuelle (tests)
```

## Client etcd v3 en Go — squelette

```go
import clientv3 "go.etcd.io/etcd/client/v3"

cli, err := clientv3.New(clientv3.Config{
    Endpoints:   []string{"localhost:2379"},
    DialTimeout: 5 * time.Second,
})
defer cli.Close()

// Lire l'état du leader actuel
resp, _ := cli.Get(context.Background(), "/leader")

// Watcher les changements (détection de panne)
watchChan := cli.Watch(context.Background(), "/leader")
for wresp := range watchChan {
    for _, ev := range wresp.Events {
        slog.Info("leader change", "key", string(ev.Kv.Key), "value", string(ev.Kv.Value))
    }
}
```

## Pièges courants

- Confondre le rôle d'etcd (stocker l'état, faire le consensus) et celui de Patroni (décider quoi faire de cet état — promouvoir, fencer). Vous n'implémentez pas Raft vous-même.
- WAL bloat : sans borne, la réplication peut accumuler jusqu'à 200 Go de WAL orphelin. Paramétrer `max_slot_wal_keep_size` dès le début, pas après coup.
- Tester le failover sans simuler une vraie coupure réseau (juste tuer le process ne teste pas le split-brain) — utilisez `iptables`/network namespaces ou testcontainers avec des toxics réseau.

## Vérifier la version

- `go.etcd.io/etcd/client/v3` dans `go.mod` : garder l'alignement avec l'etcd réellement déployé (actuellement v3.5.x en Docker). Le client v3 n'est pas compatible API-call avec le v2. L'API `etcdctl` diffère entre v2 et v3 — utiliser `ETCDCTL_API=3` ou les sous-commandes v3 (`etcdctl get`).

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc Patroni : `patroni.readthedocs.io`
- Doc etcd : `etcd.io/docs`