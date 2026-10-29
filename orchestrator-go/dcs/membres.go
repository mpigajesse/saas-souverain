// Package dcs lit l'état que Patroni publie dans etcd (son DCS) : la liste
// des membres PostgreSQL du cluster et leurs adresses. Partagé par le
// superviseur (sondes REST) et par la génération de la configuration HAProxy.
package dcs

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"net/url"
	"sort"
	"strings"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// Membre est un membre PostgreSQL enregistré par Patroni dans etcd.
type Membre struct {
	Nom    string // nom Patroni (nom de la machine)
	HotePG string // hôte:port PostgreSQL annoncé (conn_url)
	URLAPI string // base de l'API REST Patroni, sans chemin (http://hôte:port)
}

// PrefixeMembres est la clé etcd sous laquelle Patroni enregistre ses membres.
func PrefixeMembres(scope string) string { return "/service/" + scope + "/members/" }

// Membres lit les membres du scope Patroni, triés par nom. Une entrée
// illisible est ignorée : un membre mal formé ne doit pas masquer les autres.
func Membres(ctx context.Context, cli *clientv3.Client, scope string) ([]Membre, error) {
	prefixe := PrefixeMembres(scope)
	resp, err := cli.Get(ctx, prefixe, clientv3.WithPrefix())
	if err != nil {
		return nil, err
	}
	membres := make([]Membre, 0, len(resp.Kvs))
	for _, kv := range resp.Kvs {
		nom := strings.TrimPrefix(string(kv.Key), prefixe)
		m, err := DecoderMembre(nom, kv.Value)
		if err != nil {
			continue
		}
		membres = append(membres, m)
	}
	sort.Slice(membres, func(i, j int) bool { return membres[i].Nom < membres[j].Nom })
	return membres, nil
}

// Primaire renvoie le membre qui détient la clé de leader Patroni. Erreur s'il
// n'y a pas de leader (élection en cours) ou s'il est absent des membres.
func Primaire(ctx context.Context, cli *clientv3.Client, scope string) (Membre, error) {
	resp, err := cli.Get(ctx, "/service/"+scope+"/leader")
	if err != nil {
		return Membre{}, err
	}
	if resp.Count == 0 {
		return Membre{}, fmt.Errorf("aucun leader Patroni dans %s", scope)
	}
	leader := string(resp.Kvs[0].Value)
	membres, err := Membres(ctx, cli, scope)
	if err != nil {
		return Membre{}, err
	}
	for _, m := range membres {
		if m.Nom == leader {
			return m, nil
		}
	}
	return Membre{}, fmt.Errorf("leader %s absent des membres Patroni", leader)
}

// DecoderMembre extrait les adresses de la valeur JSON publiée par Patroni,
// par exemple {"conn_url": "postgres://10.0.0.5:5432/postgres",
// "api_url": "http://10.0.0.5:8008/patroni", ...}.
func DecoderMembre(nom string, valeur []byte) (Membre, error) {
	if nom == "" || strings.Contains(nom, "/") {
		return Membre{}, fmt.Errorf("nom de membre invalide : %q", nom)
	}
	var v struct {
		ConnURL string `json:"conn_url"`
		APIURL  string `json:"api_url"`
	}
	if err := json.Unmarshal(valeur, &v); err != nil {
		return Membre{}, fmt.Errorf("membre %s : %w", nom, err)
	}
	cu, err := url.Parse(v.ConnURL)
	if err != nil || cu.Host == "" {
		return Membre{}, fmt.Errorf("membre %s : conn_url invalide %q", nom, v.ConnURL)
	}
	au, err := url.Parse(v.APIURL)
	if err != nil || au.Host == "" || au.Scheme == "" {
		return Membre{}, fmt.Errorf("membre %s : api_url invalide %q", nom, v.APIURL)
	}
	hote := cu.Host
	if _, _, err := net.SplitHostPort(hote); err != nil {
		hote = net.JoinHostPort(hote, "5432")
	}
	return Membre{Nom: nom, HotePG: hote, URLAPI: au.Scheme + "://" + au.Host}, nil
}
