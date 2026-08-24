// Package mesh génère la configuration WireGuard intra-site (Jalon 4).
//
// Usage strictement intra-site dans Amane : etcd, Patroni, WAL circulent sur le
// tunnel ; le trafic inter-site passe par TLS sortant (jamais WireGuard).
//
// Règles de sécurité appliquées :
//   - AllowedIPs toujours réduit à l'IP virtuelle stable du pair (/32) — jamais
//     un routage global 0.0.0.0/0 ou ::/0 qui détournerait tout le trafic.
//   - PersistentKeepalive 25 s : indispensable derrière NAT/CGNAT.
//   - Clé privée écrite uniquement dans le fichier .conf, permissions 0600,
//     jamais loggée ni exposée (règle transverse : pas de clé en clair).
package mesh

import (
	"errors"
	"fmt"
	"os"
	"strings"
)

// DefaultListenPort est le port UDP WireGuard du mesh Amane.
const DefaultListenPort = 51820

// DefaultKeepAlive est le PersistentKeepalive (secondes) du mesh Amane.
const DefaultKeepAlive = 25

// ErrDangerousAllowedIPs : AllowedIPs trop large (route globale) refusée.
var ErrDangerousAllowedIPs = errors.New("AllowedIPs trop large : routage global interdit")

// Node décrit la machine locale du mesh.
type Node struct {
	Name       string // identifiant stable, ex. "node-1"
	Site       string // lettre du site : A, B, ... (octet du sous-réseau 10.10.<n>.0/24)
	Index      int    // dernier octet de l'IP virtuelle stable (1..254)
	PrivateKey string // clé privée du nœud — jamais loggée, jamais marshallée
	ListenPort int    // 0 → DefaultListenPort
}

// Address retourne l'IP virtuelle stable avec son masque /24.
// L'IP ne change pas quand le rôle (primary/standby) du nœud change :
// aucune reconfiguration réseau nécessaire au failover.
func (n Node) Address() (string, error) {
	octet, err := siteOctet(n.Site)
	if err != nil {
		return "", err
	}
	if n.Index < 1 || n.Index > 254 {
		return "", fmt.Errorf("Index invalide: %d (1..254)", n.Index)
	}
	return fmt.Sprintf("10.10.%d.%d/24", octet, n.Index), nil
}

// Peer décrit un pair distant du mesh.
type Peer struct {
	PublicKey string // clé publique du pair (seule la clé publique circule)
	Site      string // lettre du site (dérivation de l'IP virtuelle)
	Index     int    // dernier octet de l'IP virtuelle du pair
	Endpoint  string // host:port UDP du pair, ex. "192.168.1.20:51820"
}

// Config regroupe la machine locale et ses pairs pour un nœud du mesh.
type Config struct {
	Node  Node
	Peers []Peer
}

// ipVirtuelle retourne l'IP virtuelle /32 du pair (AllowedIPs ciblé).
func (p Peer) ipVirtuelle() (string, error) {
	octet, err := siteOctet(p.Site)
	if err != nil {
		return "", err
	}
	if p.Index < 1 || p.Index > 254 {
		return "", fmt.Errorf("Index invalide: %d (1..254)", p.Index)
	}
	return fmt.Sprintf("10.10.%d.%d/32", octet, p.Index), nil
}

// siteOctet convertit la lettre de site en octet du sous-réseau (A=1).
func siteOctet(site string) (int, error) {
	site = strings.ToUpper(site)
	if len(site) != 1 || site[0] < 'A' || site[0] > 'Z' {
		return 0, fmt.Errorf("site invalide: %q (attendu A..Z)", site)
	}
	return int(site[0]-'A') + 1, nil
}

// Render compose le contenu du fichier wg0.conf pour ce nœud.
func (c Config) Render() (string, error) {
	if c.Node.PrivateKey == "" {
		return "", errors.New("PrivateKey vide — clé privée requise")
	}
	addr, err := c.Node.Address()
	if err != nil {
		return "", err
	}

	var b strings.Builder
	b.WriteString("[Interface]\n")
	fmt.Fprintf(&b, "PrivateKey = %s\n", c.Node.PrivateKey)
	fmt.Fprintf(&b, "Address = %s\n", addr)
	port := c.Node.ListenPort
	if port == 0 {
		port = DefaultListenPort
	}
	fmt.Fprintf(&b, "ListenPort = %d\n", port)

	for _, p := range c.Peers {
		if p.PublicKey == "" {
			return "", errors.New("PublicKey de pair vide")
		}
		allowed, err := p.ipVirtuelle()
		if err != nil {
			return "", err
		}
		if isGlobal(allowed) {
			return "", fmt.Errorf("%w: %s", ErrDangerousAllowedIPs, allowed)
		}
		fmt.Fprintf(&b, "\n[Peer]\nPublicKey = %s\nAllowedIPs = %s\n", p.PublicKey, allowed)
		if p.Endpoint != "" {
			fmt.Fprintf(&b, "Endpoint = %s\n", p.Endpoint)
		}
		fmt.Fprintf(&b, "PersistentKeepalive = %d\n", DefaultKeepAlive)
	}
	return b.String(), nil
}

// ValidateAllowedIPs rejette toute règle de routage globale dans un AllowedIPs
// WireGuard (0.0.0.0/0, ::/0 ou préfixe < 16).
func ValidateAllowedIPs(ip string) error {
	if isGlobal(ip) {
		return fmt.Errorf("%w: %s", ErrDangerousAllowedIPs, ip)
	}
	return nil
}

func isGlobal(ip string) bool {
	ip = strings.TrimSpace(ip)
	switch {
	case ip == "0.0.0.0/0", ip == "::/0":
		return true
	}
	return false
}

// WriteConf écrit le fichier de configuration avec les permissions 0600
// (la clé privée ne doit jamais être lisible par un tiers).
func WriteConf(path, content string) error {
	return os.WriteFile(path, []byte(content), 0o600)
}
