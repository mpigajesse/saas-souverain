---
name: mission-c-overview
description: Entry point for the Amane Mission C skill set. Use when you need the global picture of Mission C (orchestrator-go): milestones, development order, and the index of the mission-c-* skills.
---

# Skills Mission C — Amane
Fiches de référence courtes, organisées par jalon de développement. Chaque fiche couvre : pourquoi c'est nécessaire, les concepts clés, des commandes/snippets prêts à l'emploi, les pièges courants.

| # | Fiche | Jalon | Priorité |
|---|---|---|---|
| 01 | [Go + gRPC](../mission-c-go-grpc/SKILL.md) | 1 — Contrat .proto + serveur squelette | Immédiate |
| 05 | [Docker & Tests d'intégration](../mission-c-docker-tests/SKILL.md) | Transverse | Immédiate |
| 07 | [Logging & Observabilité](../mission-c-observability/SKILL.md) | Transverse | Immédiate |
| 08 | [ProtoBuf & Buf — contrat B ↔ C](../mission-c-protobuf-buf/SKILL.md) | 1 — contrat `proto/framework.proto` | Immédiate |
| 09 | [Sécurité réseau : TLS 1.3 / mTLS / DNSSEC](../mission-c-tls-security/SKILL.md) | Transverse | Dès le jalon 1 |
| 06 | [Interface 1 (A ↔ C) : gRPC avant cgo](../mission-c-interface-a-c/SKILL.md) | Interface 1 (A ↔ C) | Dès le jalon 1 en parallèle |
| 02 | [etcd + Patroni](../mission-c-etcd-patroni/SKILL.md) | 2 — Consensus | Bientôt |
| 03 | [WireGuard & Réseaux](../mission-c-wireguard-mesh/SKILL.md) | 3 — Mesh intra-site | Plus tard |
| 04 | [CRDT](../mission-c-crdt/SKILL.md) | 4 — Réplication multi-site | Plus tard |
| 10 | [Failover & Tests HA — prouver le < 5s](../mission-c-failover-ha/SKILL.md) | 2 à 4 — mesure des jalons précédents | Bientôt |

## Ordre d'attaque conseillé

1. Commencez par **01**, **05** et **08** en parallèle — le contrat `.proto` (08) précède le serveur gRPC (01), et l'environnement Docker (05) sert de base à tout.
2. Ajoutez **07** tôt et **09** dès le squelette — instrumentez et chiffrez dès la première ligne de code, pas après coup.
3. **06** dès que vous coordonnez avec Mission A sur l'Interface 1 (via gRPC, pas cgo en premier choix).
4. **02**, **03**, **04** viennent dans l'ordre de vos jalons ; **10** dès que le consensus tourne pour mesurer le failover — pas la peine de les maîtriser avant d'y arriver.

## Note

Ces fiches viennent de connaissances générales (pas de recherche web live dans cette conversation) — **vérifiez les liens et versions de bibliothèques au moment de vous en servir**, l'écosystème Go bouge vite. Chaque fiche mentionne une section "Vérifier la version" pour les points les plus dérivants (buf, plugins protoc, client etcd, testcontainers, toxiproxy).