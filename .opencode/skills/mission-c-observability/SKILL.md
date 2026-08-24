---
name: mission-c-observability
description: Use when instrumenting Mission C Go services: log/slog structured logging, failover/lease timing correlation, OpenTelemetry forward path.
---

# Skill : Logging structuré & Observabilité
**Jalon concerné :** transverse, mais critique dès le jalon 1 — vous en aurez besoin pour débugger le failover et la réplication où le timing compte.
**Pourquoi :** un failover se joue en quelques secondes (3-5s visé) — sans logs structurés et horodatés précisément, impossible de reconstituer ce qui s'est passé.

## Concepts clés

- **`log/slog`** (stdlib Go depuis 1.21) : logging structuré natif, pas besoin de dépendance externe pour démarrer.
- **Champs structurés plutôt que texte libre** : `slog.Info("lease expired", "node", nodeID, "ttl", ttl)` plutôt que `log.Printf("lease expired for %s", nodeID)` — vous pourrez filtrer/grepper après coup.
- **Corrélation d'événements** : dans un système distribué, un même événement (ex. failover) génère des logs sur plusieurs machines — un identifiant de corrélation (trace ID) permet de les relier.
- **Niveaux** : `Debug` (verbeux, dev only), `Info` (événements normaux), `Warn` (dégradé mais fonctionnel), `Error` (échec réel).

## Squelette de base

```go
import "log/slog"

logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
    Level: slog.LevelInfo,
}))
slog.SetDefault(logger)

// Usage
slog.Info("lease renewed", "node_id", nodeID, "lease_ttl_s", 30)
slog.Warn("lease renewal delayed", "node_id", nodeID, "delay_ms", 850)
slog.Error("failover triggered", "old_primary", oldNode, "new_primary", newNode, "detection_ms", 4200)
```

## Ce qui mérite d'être loggé en priorité pour Mission C

- Chaque transition d'état Patroni (primary → standby, standby → primary)
- Chaque expiration/renouvellement de lease etcd, avec timestamp précis
- Chaque tentative de connexion WireGuard échouée (diagnostic NAT/firewall)
- Chaque conflit CRDT détecté ET sa résolution (traçabilité pour audit)
- Latence de chaque étape du chemin d'écriture (Interface 3) — utile pour vérifier vos métriques cibles (<5ms écriture locale, 50-500ms fenêtre de cohérence)

## Pièges courants

- Logger une clé, un token ou une DEK en clair "pour debug" — à bannir absolument, même en dev (habitude à ne jamais prendre).
- Logs non structurés en prod → impossible à interroger à l'échelle quand vous aurez plusieurs sites qui remontent des logs.
- Oublier de logger les cas de succès, pas juste les erreurs — pour un failover, savoir que "tout s'est bien passé en 3.2s" est aussi important que de savoir quand ça rate.

## Vérifier la version

- `log/slog` est stdlib (Go ≥ 1.21) — stable. L'Observability externe (OpenTelemetry) évolue vite : pas d'engagement avant d'en voir le besoin.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc `log/slog` : `pkg.go.dev/log/slog`
- Pour aller plus loin en observabilité multi-site : OpenTelemetry (`opentelemetry.io`) — probablement hors scope du POC mais bon à connaître pour la suite

## Trace distribuée W3C (forward path — package `telemetry/`)

Sans dépendance OTel, chaque nœud propage un **`traceparent`** W3C via le metadata gRPC et les
logs `rpc` portent `trace_id`/`span_id` : un même événement (PushDelta, Write, failover) est
corrélable d'un nœud à l'autre. Les points d'ancrage sont standard → un SDK OTel/OTLP se branche
sans modifier le code :

- `telemetry.UnaryServerInterceptor` (serveur) : extrait le `traceparent` entrant, place un
  **span enfant** dans le contexte du handler ; génère une trace racine si l'appelant n'en portait pas.
- `telemetry.UnaryClientInterceptor` (client) : injecte le `traceparent` du contexte dans le
  metadata sortant (câblé sur les connexions des propagateurs dans `cmd/orchestrator/main.go`).
- `telemetry.ParseTraceparent` / `Trace.String` / `Root` / `Child` : format W3C
  `00-<trace-id 32>-<span-id 16>-<flags 2>`, génération crypto/rand.
- `grpcserver.LoggingUnaryInterceptor` ajoute `trace_id`/`span_id` à chaque ligne `rpc`.
- Vérifier : `go test ./telemetry/ -v` (round-trip serveur↔client, racine si absent,
  parse d'en-têtes invalides) ; live : `grpcurl -plaintext -H 'traceparent: 00-...-...-01'`.
