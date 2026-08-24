---
name: mission-c-docker-tests
description: Use when setting up the Mission C local environment and integration tests: docker-compose for etcd/Patroni/PostgreSQL, testcontainers-go, cross-mission tests under tests/.
---

# Skill : Docker Compose & Tests d'intégration Go
**Jalon concerné :** transverse, mais critique dès le jalon 1 (`infra/`, `tests/`)
**Pourquoi :** vous serez probablement le principal contributeur de `infra/` — c'est vous qui montez l'environnement local (etcd + Patroni + PostgreSQL + votre service).

## Concepts clés

- **Docker Compose** pour reproduire localement un mini-cluster (3 nœuds etcd, Patroni, Postgres) sans toucher à du vrai matériel PME.
- **testcontainers-go** : lance de vrais containers (pas des mocks) depuis vos tests Go — démarre etcd/Postgres éphémères, exécute le test, détruit tout après. Beaucoup plus fiable que mocker etcd.
- **Tests d'intégration cross-missions** (`tests/`) : ceux qui valident concrètement les 3 contrats d'interface, pas juste votre code isolé.
  - Ces intégrations vivent dans `tests/` (module Go isolé `github.com/amane/tests`, `replace` vers `../orchestrator-go`).
  - Elles tournent contre etcd réel via `AMANE_TEST_ETCD` (skip sinon) — pas besoin de testcontainers pour etcd ici, la stack docker-compose partagée (etcd + Patroni) suffit.

## Squelette docker-compose.yml (base de départ)

```yaml
services:
  etcd1:
    image: quay.io/coreos/etcd:v3.5.15
    command: >
      etcd --name etcd1
           --initial-advertise-peer-urls http://etcd1:2380
           --listen-peer-urls http://0.0.0.0:2380
           --listen-client-urls http://0.0.0.0:2379
           --advertise-client-urls http://etcd1:2379
           --initial-cluster etcd1=http://etcd1:2380,etcd2=http://etcd2:2380,etcd3=http://etcd3:2380
    ports: ["2379:2379"]

  postgres:
    image: postgres:17
    environment:
      POSTGRES_PASSWORD: dev
    ports: ["5432:5432"]

  orchestrator:
    build: ./orchestrator-go
    depends_on: [etcd1, postgres]
    ports: ["50051:50051"]
```

## Exemple testcontainers-go

```go
func TestConsensusWithRealEtcd(t *testing.T) {
    ctx := context.Background()
    req := testcontainers.ContainerRequest{
        Image:        "quay.io/coreos/etcd:v3.5.15",
        ExposedPorts: []string{"2379/tcp"},
        WaitingFor:   wait.ForListeningPort("2379/tcp"),
    }
    etcdC, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
        ContainerRequest: req,
        Started:          true,
    })
    require.NoError(t, err)
    defer etcdC.Terminate(ctx)

    // ... connecter votre client etcd au container, tester consensus/
}
```

## Pièges courants

- Tester uniquement avec des mocks pour etcd/Postgres → ça passe en CI mais cache les vrais bugs de timing (leases, watch) qu'on ne voit qu'avec le vrai etcd.
- Oublier `depends_on` + `wait.For...` → les tests démarrent avant que le service soit vraiment prêt à accepter des connexions, flaky tests garantis.
- Ne pas nettoyer les containers après un test qui échoue (toujours `defer Terminate`).

## Vérifier la version

- Versions d'images : etcd (`quay.io/coreos/etcd:v3.5.x`), postgres et Patroni/Spilo bougent — vérifier à chaque mise à jour du compose. `testcontainers-go` et `wait` (pkg de waiting strategies) ont des API qui évoluent : activer/pinner les versions en CI.

## Pour aller plus loin (à vérifier, pas de recherche live)

- `testcontainers-go` : `golang.testcontainers.org`
- Docker Compose : `docs.docker.com/compose`