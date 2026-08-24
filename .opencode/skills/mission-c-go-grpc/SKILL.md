---
name: mission-c-go-grpc
description: Use when building the Amane Mission C gRPC server: grpcserver/, .proto service definitions, grpc-go interceptors, status/error codes, grpcurl for manual tests.
---

# Skill : Go + gRPC
**Jalon concerné :** 1 — Contrat `.proto` + serveur gRPC squelette
**Pourquoi :** `grpcserver/` est le seul point de contact réseau du framework Amane. C'est la première brique à faire tourner.

## Concepts clés

- **Unary RPC** : requête → une réponse (99% de vos cas au départ).
- **Streaming** : utile plus tard pour pousser l'état du cluster en continu vers un moniteur.
- **Codes d'erreur gRPC** (`codes.NotFound`, `codes.PermissionDenied`, `codes.Unavailable`...) — ne renvoyez jamais une `error` Go brute, wrappez-la avec `status.Error(codes.X, "message")`.
- **Interceptors** (middleware gRPC) : point idéal pour brancher logging, auth, et plus tard la vérification CRL côté Mission A.
- **Contexte (`context.Context`)** : chaque méthode gRPC en reçoit un — c'est là que passent deadlines et annulation. Ne jamais l'ignorer.

## Commandes essentielles

```bash
protoc --go_out=. --go_opt=paths=source_relative \
       --go-grpc_out=. --go-grpc_opt=paths=source_relative \
       proto/amane.proto

grpcurl -plaintext localhost:50051 list
grpcurl -plaintext -d '{"id": "abc"}' localhost:50051 amane.AmaneService/Enroll
```

## Squelette minimal d'un serveur

```go
package main

import (
    "context"
    "log/slog"
    "net"

    "google.golang.org/grpc"
    pb "amane/proto/gen"
)

type server struct {
    pb.UnimplementedAmaneServiceServer
}

func (s *server) Ping(ctx context.Context, req *pb.PingRequest) (*pb.PingResponse, error) {
    return &pb.PingResponse{Status: "ok"}, nil
}

func main() {
    lis, err := net.Listen("tcp", ":50051")
    if err != nil {
        slog.Error("listen failed", "err", err)
        return
    }
    s := grpc.NewServer()
    pb.RegisterAmaneServiceServer(s, &server{})
    slog.Info("grpc server listening", "addr", ":50051")
    s.Serve(lis)
}
```

## Pièges courants

- Oublier `UnimplementedXxxServer` dans votre struct → le code casse dès que le `.proto` gagne une méthode (c'est voulu, ça vous force à l'implémenter).
- Renvoyer `nil, nil` sur erreur au lieu de `nil, status.Error(...)` — le client ne verra jamais l'échec proprement.
- Bloquer indéfiniment dans une méthode sans respecter `ctx.Done()` — un client qui timeout ne libère pas la ressource côté serveur si vous l'ignorez.

## Vérifier la version

- `google.golang.org/grpc` et `google.golang.org/protobuf` dans `go.mod` : vérifier que les versions des plugins (`~/go/bin/protoc-gen-go`, `protoc-gen-go-grpc`) sont alignées avec celles du runtime, sinon le code généré ne compile pas.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc officielle gRPC-Go : `grpc.io/docs/languages/go`
- Style guide Protobuf de Google : `protobuf.dev/programming-guides/style`