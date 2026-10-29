// Package portail dialogue avec le portail de l'éditeur. Il n'échange que des
// métadonnées (ADR-003) : IP des machines déjà installées, présence d'une
// machine. Aucun secret ni donnée métier ne transite par ici.
package portail

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Client du portail, authentifié par le jeton d'enregistrement du tenant.
type Client struct {
	Base  string // ex. http://192.168.7.7:80
	Jeton string
	HTTP  *http.Client
}

// Nouveau crée un client avec un délai raisonnable pour un réseau de PME.
func Nouveau(base, jeton string) *Client {
	return &Client{Base: strings.TrimRight(base, "/"), Jeton: jeton, HTTP: &http.Client{Timeout: 10 * time.Second}}
}

// Pairs renvoie les IP des autres machines actives du tenant (serveurs et
// témoins), d'après GET /api/devices/cluster-status/.
func (c *Client) Pairs(ctx context.Context, ipSoi string) ([]string, error) {
	u := c.Base + "/api/devices/cluster-status/?token=" + url.QueryEscape(c.Jeton)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, u, nil)
	if err != nil {
		return nil, err
	}
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return nil, fmt.Errorf("portail injoignable : %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		corps, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<12))
		return nil, fmt.Errorf("portail : statut %d : %s", resp.StatusCode, strings.TrimSpace(string(corps)))
	}
	var etat struct {
		Noeuds []struct {
			NodeAddr string `json:"node_addr"`
		} `json:"active_nodes"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&etat); err != nil {
		return nil, fmt.Errorf("réponse du portail illisible : %w", err)
	}
	vus := map[string]bool{ipSoi: true, "": true}
	var ips []string
	for _, n := range etat.Noeuds {
		ip := n.NodeAddr
		if h, _, err := net.SplitHostPort(n.NodeAddr); err == nil {
			ip = h
		}
		if net.ParseIP(ip) == nil || vus[ip] {
			continue
		}
		vus[ip] = true
		ips = append(ips, ip)
	}
	sort.Strings(ips)
	return ips, nil
}

// Annonce décrit une machine qui se signale au portail.
type Annonce struct {
	InstallationID string
	Adresse        string // ip:port annoncé
	Hostname       string
	Type           string // "serveur" ou "temoin"
}

// Annoncer enregistre la machine ou rafraîchit sa présence
// (POST /api/devices/register/).
func (c *Client) Annoncer(ctx context.Context, a Annonce) error {
	corps, err := json.Marshal(map[string]string{
		"tenant_token":    c.Jeton,
		"installation_id": a.InstallationID,
		"node_addr":       a.Adresse,
		"hostname":        a.Hostname,
		"os":              "linux",
		"node_kind":       a.Type,
	})
	if err != nil {
		return err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.Base+"/api/devices/register/", bytes.NewReader(corps))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return fmt.Errorf("portail injoignable : %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		reponse, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<12))
		return fmt.Errorf("portail : statut %d : %s", resp.StatusCode, strings.TrimSpace(string(reponse)))
	}
	return nil
}

// IdentifiantInstallation lit l'UUID d'installation de la machine, ou le
// crée au premier lancement. Il identifie la machine auprès du portail
// (jamais la MAC, cf. devices/api_register.py).
func IdentifiantInstallation(chemin string) (string, error) {
	if b, err := os.ReadFile(chemin); err == nil {
		if id := strings.TrimSpace(string(b)); id != "" {
			return id, nil
		}
	} else if !errors.Is(err, os.ErrNotExist) {
		return "", err
	}
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	b[6] = b[6]&0x0f | 0x40 // UUID v4
	b[8] = b[8]&0x3f | 0x80
	id := fmt.Sprintf("%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
	if err := os.MkdirAll(filepath.Dir(chemin), 0o755); err != nil {
		return "", err
	}
	return id, os.WriteFile(chemin, []byte(id+"\n"), 0o644)
}
