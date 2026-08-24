---
name: mission-c-crdt
description: Use when implementing Mission C multi-site replication (replication/): delta-CRDT, PN-Counter for stock quantities, convergence and conflict resolution strategies.
---

# Skill : CRDT (Conflict-Free Replicated Data Types)
**Jalon concerné :** 4 — Réplication CRDT multi-site (`replication/`)
**Pourquoi :** c'est le mécanisme qui permet des écritures non-bloquantes en mode partagé AP, sans passer par un verrou distribué.

## Concepts clés

- **State-based (CvRDT)** : chaque nœud envoie son état complet, la fusion se fait par une fonction "merge" commutative/associative/idempotente. Simple à raisonner, mais coûteux en bande passante.
- **Operation-based (CmRDT)** : chaque nœud envoie l'opération elle-même (ex. "+3"), pas l'état. Plus léger sur le réseau, mais demande une livraison fiable des opérations (au moins une fois, dans un ordre causal correct).
- **Delta-CRDT** (utilisé dans Amane) : compromis — on envoie seulement le changement (delta) plutôt que l'état complet, tout en gardant les garanties de convergence du state-based.
- **Convergence** : la propriété centrale — peu importe l'ordre dans lequel les deltas arrivent, tous les nœuds finissent dans le même état.

## Rappel : ce n'est PAS une solution universelle

CRDT ne résout que les types de données pour lesquels une fusion mathématiquement propre existe. Dans Amane, seul le **stock/quantités** (compteur delta) utilise du CRDT pur. Les autres types de données partagées utilisent d'autres stratégies (LWW, arbitrage humain) — voir le tableau de résolution de conflits.

## Exemple minimal : compteur delta (le cas d'usage stock)

```go
// PN-Counter simplifié (Positive-Negative Counter)
type DeltaCounter struct {
    NodeID string
    Increments map[string]int64  // increments par nœud
    Decrements map[string]int64  // décréments par nœud
}

func (c *DeltaCounter) Value() int64 {
    var total int64
    for _, v := range c.Increments { total += v }
    for _, v := range c.Decrements { total -= v }
    return total
}

// Merge = prendre le max par nœud pour chaque compteur (idempotent, commutatif)
func Merge(a, b *DeltaCounter) *DeltaCounter {
    result := &DeltaCounter{
        Increments: make(map[string]int64),
        Decrements: make(map[string]int64),
    }
    for node, v := range a.Increments {
        result.Increments[node] = max(v, b.Increments[node])
    }
    for node, v := range b.Increments {
        if _, ok := result.Increments[node]; !ok {
            result.Increments[node] = v
        }
    }
    // même logique pour Decrements
    return result
}
```

## Relay multi-site (orchestrator-go)

La propagation inter-sites du compteur passe par `replication/relay.go` + RPC
`PushDelta` (`proto/amane/framework/v1/framework.proto`) :

- **Total cumulé, pas incrément** : un delta porte les totaux Inc/Dec du nœud émetteur
  depuis son origine → la fusion **max par nœud émetteur** est commutative, associative
  et idempotente (garantie AP).
- **seq = gc du pending, pas la convergence** : doublons, réordonnancements et trous de
  séquence ne compromettent jamais la convergence ; la seq ne sert qu'à l'ack du pair et
  au nettoyage (`Outgoing()`/`Confirm(ackSeq)`, fuite évitée).
- `Add(inc, dec)` met à jour total + pending + seq ; `Accept(fromNode, deltas)` fusionne
  côté récepteur (réflexion d'un delta auto-émis ignorée).
- Câblage : `grpcserver.WithRelay(*replication.Relay)` — handler `PushDelta`
  (validate → Accept → réponse ack seq + valeur locale) ; sans relay → `codes.Unavailable`.
- **Propagation périodique** : un `Propagator` (`replication/propagator.go`) tire le pending
  toutes les `-replicate-interval` (défaut 1 s) et pousse via `PushDeltaTransport`
  (`grpcserver/transport.go`) vers chaque pair `-replicate-to site-id@host:port` ; l'ack du pair
  = **dernière seq appliquée** (jamais 0) → `Confirm` vide le pending. Échec réseau : retentée au
  tick suivant, deltas jamais perdus.
- Tests : `go test ./replication/ -count=1 -race` (convergence 3 nœuds, ack gc, réflexion,
  trous de seq, merge associatif, propagateur push/confirm + retry sans perte) ;
  `go test ./grpcserver/ -run 'TestPushDelta|TestPropagatorE2E' -v` (2 serveurs gRPC TCP réels).

## Pièges courants

- Utiliser un CRDT "compteur simple" (sans distinction increment/decrement par nœud) → ne converge pas correctement si deux nœuds décrémentent en même temps (le stock peut devenir négatif de façon incohérente).
- Croire que CRDT élimine le besoin de tester les conflits — au contraire, écrivez des tests qui simulent explicitement des écritures concurrentes désordonnées sur 3+ nœuds (c'est l'indicateur de succès POC : "0 conflit non résolu").
- Vouloir tout faire en CRDT — relisez le tableau de stratégies par type de donnée avant de coder, certaines données (factures) ne doivent PAS être résolues automatiquement.

## Vérifier la version

- L'écosystème CRDT Go est peu mature et changeant : avant de coder, réévaluer l'état des bibliothèques existantes (le texte de ce document scolaire peut être obsolète). Les algos de base (PN-Counter, LWW) sont stables — les implémentations distribuées delta le sont moins.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Papier de référence : Shapiro et al., "A comprehensive study of Convergent and Commutative Replicated Data Types" (INRIA, 2011)
- Bibliothèques Go existantes à évaluer avant de coder à la main : rechercher l'état de l'écosystème Go CRDT au moment de démarrer ce jalon (peu mature comparé à Yjs/Automerge en JS/Rust)