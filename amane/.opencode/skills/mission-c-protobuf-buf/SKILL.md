---
name: mission-c-protobuf-buf
description: Use when maintaining the shared proto/framework.proto contract between Missions B and C: buf lint/generate/breaking, protoc-gen-go, proto versioning and compatibility.
---

# Skill : ProtoBuf & Buf — le contrat partagé B ↔ C
**Jalon concerné :** 1 — le fichier `proto/framework.proto` est le point de départ de tout
**Pourquoi :** c'est le contrat unique qui relie les Missions B (client SDK) et C (serveur gRPC). Une erreur d'évolution ici casse les deux côtés à la fois — c'est le premier fichier à poser, avant de coder `grpcserver/`.

## Concepts clés

- **proto3** : syntaxe par défaut de Protobuf. Chaque message a des **tags de champ numériques** (1, 2, 3…) — ce sont eux la "mémoire" du format, PAS les noms.
- **Service RPC** : `service AmaneService { rpc Ping(PingRequest) returns (PingResponse); }` — la définition des méthodes exposées.
- **Compatibilité** : ajouter un champ = sans danger ; renuméroter ou changer le type d'un champ = rupture silencieuse.
- **buf (build & linting)** : remplace l'usage manuel de `protoc` — `buf generate`, `buf lint`, et surtout `buf breaking` qui détecte les changements de type incompatibles avant de casser les SDKs.

## Commandes essentielles

```bash
buf generate                       # (config buf.gen.yaml dans le repo)

buf lint

buf breaking --against '.git#branch=main,subdir=proto'

buf mod update && buf build
```

## Squelette minimal `framework.proto` (à enrichir avec la Mission B)

```proto
syntax = "proto3";

package amane.framework.v1;
option go_package = "amane/proto/gen/frameworkv1";

service AmaneService {
  rpc Ping(PingRequest) returns (PingResponse);
  rpc Enroll(EnrollRequest) returns (EnrollResponse);
  rpc Write(WriteRequest) returns (WriteResponse);   // Interface 3 (chemin d'écriture)
}

message PingRequest {}
message PingResponse { string status = 1; }

message EnrollRequest {
  string machine_id = 1;
  bytes  ak_public_key = 2;   // jamais la clé privée
}
```

## Pièges courants (coûteux en re-codage)

- Renumérotter des tags ou changer le type d'un champ existant → les clients déjà déployés lisent n'importe quoi, sans aucune erreur (c'est le pire des cas : silencieux).
- Oublier la convention de nommage des packages (`amane.framework.v1`) ou du `go_package` → imports générés incohérents entre Missions B et C.
- Ne pas **commiter le code généré** dans le repo et le régénérer à la main avec des versions de plugins différentes → des builds qui divergent entre machines.
- Utiliser `enum` à choix multiples dans les messages au lieu de `oneof` quand les cas sont mutuellement exclusifs.
- Ajouter des champs "temporaires pour debug" dans le contrat → ils restent à jamais.

## Vérifier la version

- `buf` version actuelle : `buf.build` — le format de `buf.gen.yaml` et les commandes ont évolué ces dernières années. `protoc-gen-go` et `protoc-gen-go-grpc` sont installés dans `~/go/bin` ; veiller à ce que leurs versions soient alignées avec `google.golang.org/protobuf` dans `go.mod`.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc Buf : `buf.build/docs`
- Style guide Protobuf Google : `protobuf.dev/programming-guides/style`