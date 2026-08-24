---
name: mission-c-tls-security
description: Use when securing Amane networks (Mission C): TLS 1.3 minimum, mTLS for gRPC, internal PKI/CA, DNSSEC validation outbound.
---

# Skill : Sécurité réseau — TLS 1.3, mTLS, DNSSEC
**Jalon concerné :** transverse — dès le jalon 1 pour le serveur gRPC, puis mesh (03) et inter-site
**Pourquoi :** la note de synthèse Amane exige **TLS 1.3 sur toutes les sorties**, validation **DNSSEC**, et le maillage WireGuard ne suffit pas — même à l'intérieur du mesh, le trafic applicatif doit être chiffré en plus (defense in depth).

## Concepts clés

- **TLS 1.3** : version minimale acceptée (`MinVersion: tls.VersionTLS13`). Pas la négocier au niveau 1.2 "par compatibilité".
- **mTLS** : TLS mutuel — le serveur demande ET vérifie un certificat client. C'est la vraie authentification par identité de machine pour la réplication et l'admin (interface Patroni/etcd), pas un simple token.
- **PKI interne** : autorité de certification privée (CA) qui délivre les certificats des nœuds — les certificats publics des PME ne sont pas concernés.
- **DNSSEC** : le client doit **valider** les signatures DNSSEC à la résolution (pas juste demander `dnssec`) — sinon la sortie vers les services cloud peut être détournée.
- **Frontière avec Mission A** : tout ce qui est chiffré/dé-chiffré par `ss-crypto` reste côté A. Côté Go, on ne manipule que des clés publiques et des données chiffrées (jamais la DEK/AK privée en clair).

## gRPC + mTLS (squelette serveur)

```go
import (
    "crypto/tls"
    "crypto/x509"
    "google.golang.org/grpc/credentials"
)

// Côté serveur : exiger un certificat client valide
pool := x509.NewCertPool()
pool.AppendCertsFromPEM(caPEM) // CA interne

creds := credentials.NewTLS(&tls.Config{
    Certificates: []tls.Certificate{serverCert},
    ClientCAs:    pool,
    ClientAuth:   tls.RequireAndVerifyClientCert, // = mTLS
    MinVersion:   tls.VersionTLS13,
})

s := grpc.NewServer(grpc.Creds(creds))
```

## Snippets utiles

```bash
openssl req -new -newkey rsa:3072 -nodes \
    -keyout node.key -out node.csr -subj "/CN=node-1.amane.internal"
openssl x509 -req -in node.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out node.crt -days 365

openssl s_client -connect host:443 -tls1_2 2>&1 | head -3   # doit échouer au handshake
```

## Pièges courants

- Laisser `MinVersion` par défaut → le serveur accepte TLS 1.2, ce qui viole l'exigence et ouvre des failles connues si on régresse.
- mTLS mal configuré : vérifier le certificat client (`RequestClientCert` **sans** `RequireAndVerify`) → personne n'est réellement authentifié.
- **Certificats auto-signés sans CA partagée** : chaque nœud qui "fait confiance à tout" (`InsecureSkipVerify: true`) détruit tout le modèle.
- Une clé privée ou une DEK qui traîne dans un log, un dump mémoire Go, ou un variable d'environnement — vérifier avec la fiche 07 que rien de sensible ne sort en Go.

## Vérifier la version

- API `crypto/tls` est stable en stdlib — la source de dérive est dans les plugins gRPC (`credentials`/`x509`), vérifier l'alignement avec `google.golang.org/grpc` de `go.mod`. Vérifier l'état de l'art DNSSEC côté résolveur (systemd-resolved / stubby / dnsmasq) selon la distrib.

## Pour aller plus loin (à vérifier, pas de recherche live)

- mTLS gRPC : `grpc.io/docs/guides/auth/` (§ "Authenticate with TLS")
- Configuration DNSSEC côté client résolveur : docs `systemd-resolved` / `stubby`