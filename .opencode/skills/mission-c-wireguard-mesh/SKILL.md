---
name: mission-c-wireguard-mesh
description: Use when setting up the Amane intra-site WireGuard mesh (mesh/): wg/wg-quick config, AllowedIPs, NAT/CGNAT, PersistentKeepalive. Not for inter-site traffic.
---

# Skill : WireGuard & Réseaux
**Jalon concerné :** 3 — Réseau maillé WireGuard (`mesh/`)
**Pourquoi :** usage strictement intra-site dans Amane V2 (etcd, Patroni, WAL) — plus jamais entre deux sites (remplacé par TLS sortant vers le Blind Relay).

## Concepts clés

- **Clé publique/privée par pair** — chaque machine a sa paire, pas de mot de passe partagé.
- **AllowedIPs** : détermine quel trafic passe par le tunnel pour un pair donné — mal configuré, ça peut soit tout router (dangereux), soit rien router (tunnel inutile).
- **PersistentKeepalive** (25s dans Amane) : nécessaire pour maintenir un tunnel ouvert derrière du NAT — sans ça, le NAT ferme la session après inactivité.
- **CGNAT** : le vrai problème que WireGuard mesh rencontre en usage inter-site en Afrique — deux machines derrière un CGNAT ne peuvent pas s'atteindre directement, d'où le choix Amane de connexions sortantes TLS pour l'inter-site.
- **IP virtuelle stable** : dans Amane, l'IP WireGuard d'une machine ne change pas même si son rôle (primary/standby) change — évite toute reconfiguration réseau au failover.

## Commandes essentielles

```bash
wg genkey | tee privatekey | wg pubkey > publickey


wg-quick up wg0
wg show                 # état des tunnels, dernier handshake
wg-quick down wg0

ping 10.10.A.2
```

## Exemple de configuration (une machine du mesh intra-site)

```ini
[Interface]
PrivateKey = <clé privée machine 1>
Address = 10.10.A.1/24
ListenPort = 51820

[Peer]
PublicKey = <clé publique machine 2>
AllowedIPs = 10.10.A.2/32
Endpoint = 192.168.1.20:51820
PersistentKeepalive = 25

[Peer]
PublicKey = <clé publique VPS>
AllowedIPs = 10.10.A.3/32
Endpoint = vps.example.com:51820
PersistentKeepalive = 25
```

## Runtime (publication etcd + découverte — `mesh/runtime.go` + `cmd/wgmesh`)

Chaque nœud publie son **information publique** (nom, site, index, clé publique, endpoint UDP)
sous `/amane/mesh/nodes/<name>` dans etcd — **jamais la clé privée**, qui reste locale (fichier
0600). `wgmesh` découvre ses pairs, régénère `wg0.conf` (0600) à chaque changement de membres et
ne réécrit que si le contenu diffère ; `wg-quick up/down` reste l'application du tunnel.

```bash
go build -o /tmp/wgmesh ./cmd/wgmesh && /tmp/wgmesh -etcd localhost:2379 -name node-1 -site A \
  -index 1 -pubkey <clé publique> -privkey-file /path/to/wg_private.key \
  -endpoint 192.168.1.20:51820 -conf /tmp/wg0-node1.conf -interval 1s
```

Tests : `go test ./mesh/ -count=1 -race` (registration/découverte excluant soi-même, PeerInfo
validée, 0600 + skip si inchangé, jamais de routage global, RunSync régénère à l'arrivée d'un pair).

## Pièges courants

- Mettre `AllowedIPs = 0.0.0.0/0` par erreur → route TOUT le trafic de la machine dans le tunnel, pas juste le trafic destiné au mesh.
- Oublier `PersistentKeepalive` sur les machines derrière NAT → le tunnel semble fonctionner puis se coupe après quelques minutes d'inactivité.
- Tester uniquement en local (même réseau) sans jamais simuler un vrai NAT/firewall — ça cache les vrais problèmes de prod.

## Vérifier la version

- `wireguard-tools` (wg/wg-quick) selon la distrib : noyau ≥ 5.6 requis sinon module manquant. La bibliothèque Go `golang.zx2c4.com/wireguard/wgctrl` peut évoluer avec les versions du kernel — vérifier la compatibilité au moment d'automatiser `mesh/`.

## Pour aller plus loin (à vérifier, pas de recherche live)

- Doc officielle WireGuard : `wireguard.com/quickstart`
- `wgctrl-go` (bibliothèque Go pour piloter WireGuard programmatiquement, utile pour automatiser la config depuis `mesh/`)