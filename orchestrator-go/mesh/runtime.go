package mesh

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// peerKeyPrefix est le préfixe etcd d'enregistrement des membres du mesh :
// /amane/mesh/nodes/<name>. Seule l'information PUBLIQUE y circule (jamais la
// clé privée — règle transverse).
const peerKeyPrefix = "/amane/mesh/nodes/"

// PeerInfo est l'information publique qu'un nœud publie pour être découvert
// par ses pairs (site, index, clé publique, endpoint). Pas de clé privée.
type PeerInfo struct {
	Name      string `json:"name"`
	Site      string `json:"site"`  // lettre du site A..Z
	Index     int    `json:"index"` // dernier octet de l'IP virtuelle (1..254)
	PublicKey string `json:"public_key"`
	Endpoint  string `json:"endpoint,omitempty"` // host:port UDP (absent si inconnu)
}

// KV abstrait l'accès etcd pour la publication/découverte (testable sans etcd).
type KV interface {
	Put(ctx context.Context, key, value string) error
	List(ctx context.Context, prefix string) (map[string]string, error)
}

// EtcdKV adapte *clientv3.Client (etcd réel) à l'interface KV.
type EtcdKV struct{ Cli *clientv3.Client }

func (k EtcdKV) Put(ctx context.Context, key, value string) error {
	_, err := k.Cli.Put(ctx, key, value)
	return err
}

func (k EtcdKV) List(ctx context.Context, prefix string) (map[string]string, error) {
	resp, err := k.Cli.Get(ctx, prefix, clientv3.WithPrefix())
	if err != nil {
		return nil, err
	}
	out := make(map[string]string, resp.Count)
	for _, kv := range resp.Kvs {
		out[string(kv.Key)] = string(kv.Value)
	}
	return out, nil
}

// Register publie l'information publique du nœud local (aucun secret). Le
// timestamp permet de détecter les membres morts (lease courte à venir).
func Register(ctx context.Context, store KV, info PeerInfo) error {
	if err := ValidatePeerInfo(info); err != nil {
		return err
	}
	b, err := json.Marshal(info)
	if err != nil {
		return err
	}
	return store.Put(ctx, peerKeyPrefix+info.Name, string(b))
}

// ValidatePeerInfo vérifie la cohérence d'une PeerInfo avant publication.
func ValidatePeerInfo(info PeerInfo) error {
	if info.Name == "" {
		return errors.New("nom du nœud requis")
	}
	if _, err := siteOctet(info.Site); err != nil {
		return err
	}
	if info.Index < 1 || info.Index > 254 {
		return errors.New("index invalide (1..254)")
	}
	if len(info.PublicKey) < 32 {
		return errors.New("clé publique WireGuard invalide")
	}
	return nil
}

// DiscoverPeers liste les pairs du mesh, en excluant le nœud local (selfName).
// Un pair au registre malformé est ignoré (et remonté dans l'erreur partielle).
func DiscoverPeers(ctx context.Context, store KV, selfName string) ([]Peer, error) {
	entries, err := store.List(ctx, peerKeyPrefix)
	if err != nil {
		return nil, err
	}
	var peers []Peer
	for key, raw := range entries {
		var info PeerInfo
		if err := json.Unmarshal([]byte(raw), &info); err != nil {
			return nil, fmt.Errorf("pair %q malformé: %w", key, err)
		}
		if info.Name == selfName {
			continue
		}
		peers = append(peers, Peer{
			PublicKey: info.PublicKey,
			Site:      info.Site,
			Index:     info.Index,
			Endpoint:  info.Endpoint,
		})
	}
	return peers, nil
}

// GenerateConfig produit le wg0.conf du nœud local et l'écrit en 0600 s'il a
// changé. Retourne (changement détecté, chemin, erreur) pour faciliter le
// rechargement par l'opérateur (wg-quick down/up).
func GenerateConfig(confPath string, local Node, peers []Peer) (bool, error) {
	cfg := Config{Node: local, Peers: peers}
	rendered, err := cfg.Render()
	if err != nil {
		return false, err
	}
	existing, readErr := readFileIfExists(confPath)
	if readErr == nil && existing == rendered {
		return false, nil
	}
	if err := WriteConf(confPath, rendered); err != nil {
		return false, err
	}
	return true, nil
}

func readFileIfExists(path string) (string, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// RunSync boucle la découverte + génération à intervalle fixe : chaque nœud du
// mesh voit les pairs arriver/partir et sa conf est régénérée sans secret.
func RunSync(ctx context.Context, store KV, selfName string, local Node, confPath string, interval time.Duration) error {
	syncOnce := func() (bool, error) {
		peers, err := DiscoverPeers(ctx, store, selfName)
		if err != nil {
			return false, err
		}
		return GenerateConfig(confPath, local, peers)
	}
	if changed, err := syncOnce(); err != nil {
		return fmt.Errorf("génération initiale de la conf mesh: %w", err)
	} else if changed {
		fmt.Println("mesh: wg0.conf régénéré (nouveaux pairs)")
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			changed, err := syncOnce()
			if err != nil {
				return err
			}
			if changed {
				fmt.Println("mesh: wg0.conf régénéré (membres du mesh changés)")
			}
		}
	}
}
