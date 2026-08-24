---
name: mission-c-interface-a-c
description: Use when coordinating Mission C with Mission A (Interface 1): membership/enrollment, revocation notification via gRPC (NotifyRevocation), cgo only as last resort, never exposing keys in clear to Go.
---

# Skill : Interface 1 (A ↔ C) — gRPC en priorité, cgo en dernier recours
**Jalon concerné :** en lien avec l'Interface 1 (A ↔ C) — enrôlement & membership
**Pourquoi :** quand `consensus/` doit savoir qu'une machine a été dé-enrôlée (AK révoquée dans la CRL côté Mission A), il faut que **C soit notifié** par A. La voie normale est un appel réseau, pas un appel de fonction local.

## Concept clé à bien comprendre

Vous n'écrivez **aucune cryptographie**. Tout ce qui touche les clés vit dans `ss-crypto` (Mission A). Côté Go, on ne manipule que des identifiants (machine_id), des clés **publiques** et des statuts — jamais la AK privée ni la DEK.

## Approche par défaut : notification gRPC (choisie)

L'Interface 1 peut se coder dans le `framework.proto` partagé : A (via le client de son SDK) appelle une RPC que C expose, par exemple `rpc NotifyRevocation(NotifyRevocationRequest)` — C recoît le `machine_id` révoqué, déclenche immédiatement le recalcul du quorum dans `consensus/`.

C'est cohérent avec :
- la frontière "aucun clair vers Go" de la Mission A (seul un **statut/ID** traverse),
- l'architecture monorepo existante (le `proto/` est le point de contact B ↔ C),
- le découplage des langages — pas de lien de build entre Rust et Go.

## Quand cgo devient envisageable (dernier recours)

Un appel local à la bibliothèque Rust n'a de sens que pour un cas **mesuré à fort coût réseau** (ex. vérifier une signature à chaud sur chaque requête) — ET à condition d'avoir tranché avec Mission A. Même là, deux alternatives plus sûres que cgo :

1. **Processus séparé + IPC local** (socket Unix, gRPC loopback) : isole un crash Rust d'un crash Go, limite la surface mémoire partagée.
2. **Hors ligne/precomputation** : faire vérifier les blobs de confiance par A en amont (par ex. à l'enrôlement), pas à chaque requête.

## Squelette cgo minimal (approche 2, pour comprendre le mécanisme)

```go
package main

/*
#cgo LDFLAGS: -L. -lsscrypto
#include <stdint.h>

extern int verify_ak_token(const char* token, size_t len);
*/
import "C"
import "unsafe"

func VerifyAKToken(token []byte) bool {
    cToken := C.CBytes(token)
    defer C.free(cToken)
    result := C.verify_ak_token((*C.char)(cToken), C.size_t(len(token)))
    return result == 1
}
```

Côté Rust (Mission A expose une fonction `extern "C"`) :

```rust
#[no_mangle]
pub extern "C" fn verify_ak_token(token: *const u8, len: usize) -> i32 {
    // ... vérification Ed25519, retourne 1 ou 0
}
```

## Pièges courants

- Passer une clé privée ou une DEK en clair à travers une frontière (gRPC ou cgo) → violation directe de l'Interface 2 de la Mission A.
- Gestion mémoire cgo : toute donnée passée à la frontière doit avoir un propriétaire clair — qui libère quoi, et quand. Préférez que Rust retourne un simple booléen/statut plutôt que la donnée sensible.
- cgo désactive certaines optimisations de compilation croisée Go — impact possible sur le pipeline CI multi-plateforme ; à ne pas prendre à la légère.
- Confondre "notifier C" (Interface 1) et "vérifier le chiffrement" (Interface 2) : le chemin d'écriture complet (Interface 3) passe par le journal chiffré de A, pas par des appels ad-hoc en clair.

## Vérifier la version

- cgo : `pkg.go.dev/cmd/cgo` ; Rust FFI : `doc.rust-lang.org/nomicon/ffi.html`. Vérifier la convention de nommage entre la crate `ss-crypto` de Mission A et la signature `extern "C"` attendue côté Go avant de figer le contrat.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc cgo : `pkg.go.dev/cmd/cgo`
- Rust FFI : chapitre "FFI" du Rust Nomicon, ou `doc.rust-lang.org/nomicon/ffi.html`