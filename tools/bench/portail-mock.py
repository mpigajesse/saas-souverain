#!/usr/bin/env python3
"""Portail éditeur factice pour le banc de VM AMANE (tâche A-00).

Le banc est isolé du vrai portail (SAAS_URL du banc pointe ici, sur m1).
Ce serveur imite strictement les deux appels que la pile PME fait au portail
(orchestrator-go/portail/portail.go) :

  GET  /api/devices/cluster-status/?token=…
       -> {"active_nodes": [{"node_addr": "ip:port"}, …]}
          = les machines « déjà installées » du tenant ; amane-init s'en sert
            pour décider de créer ou rejoindre le cluster etcd.
  POST /api/devices/register/  (tenant_token, installation_id, node_addr, …)
       -> enregistre la machine et répond 200.

L'état (liste des machines) est lu à chaque requête dans un fichier JSON que
le déploiement met à jour entre deux amorçages, ce qui rend l'ordre de
création/jonction du cluster etcd déterministe :

  {}                            -> m1 crée le cluster
  {"10.77.0.11"}                -> m2 le rejoint
  {"10.77.0.11","10.77.0.12"}   -> m3 le rejoint

Toute requête est journalisée (preuve des échanges pile <-> portail).
Usage : portail-mock.py --port 5173 --state etat.json --log serveur.log
"""
import argparse
import json
import os
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse

verrou = threading.Lock()
ARGS = None


def lire_etat() -> dict:
    try:
        with open(ARGS.state, encoding="utf-8") as f:
            etat = json.load(f)
    except (OSError, ValueError):
        etat = {}
    if "active_nodes" not in etat:
        etat = {"active_nodes": list(etat.get("active_nodes", []))}
    return etat


def ecrire_etat(etat: dict) -> None:
    tmp = ARGS.state + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(etat, f, indent=1)
    os.replace(tmp, ARGS.state)


class Handler(BaseHTTPRequestHandler):
    def _repondre(self, code: int, corps: dict) -> None:
        corps = json.dumps(corps).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(corps)))
        self.end_headers()
        self.wfile.write(corps)

    def do_GET(self) -> None:
        u = urlparse(self.path)
        if u.path == "/api/devices/cluster-status/":
            with verrou:
                etat = lire_etat()
            self._repondre(200, etat)
            self._journal(f"cluster-status -> {etat}")
            return
        self._repondre(200, {"ok": True, "service": "portail-factice-banc"})
        self._journal(f"GET {u.path} -> 200 générique")

    def do_POST(self) -> None:
        u = urlparse(self.path)
        taille = int(self.headers.get("Content-Length") or 0)
        corps = self.rfile.read(taille).decode("utf-8", "replace")
        if u.path == "/api/devices/register/":
            try:
                annonce = json.loads(corps)
            except ValueError:
                annonce = {}
            adresse = annonce.get("node_addr", "")
            if adresse:
                with verrou:
                    etat = lire_etat()
                    noeuds = [n.get("node_addr") for n in etat.get("active_nodes", [])]
                    if adresse not in noeuds:
                        etat["active_nodes"].append({"node_addr": adresse})
                    ecrire_etat(etat)
            self._repondre(200, {"ok": True})
            self._journal(f"register <- {adresse}")
            return
        self._repondre(404, {"erreur": "chemin inconnu"})
        self._journal(f"POST {u.path} -> 404")

    def _journal(self, message: str) -> None:
        ligne = f"{time.strftime('%Y-%m-%d %H:%M:%S')} {message}\n"
        with verrou:
            try:
                with open(ARGS.log, "a", encoding="utf-8") as f:
                    f.write(ligne)
            except OSError:
                pass

    def log_message(self, *args) -> None:  # silencieux : journalisé ailleurs
        pass


def main() -> None:
    global ARGS
    p = argparse.ArgumentParser()
    p.add_argument("--port", type=int, default=5173)
    p.add_argument("--state", default="/home/etudiant/portail/etat.json")
    p.add_argument("--log", default="/home/etudiant/portail/serveur.log")
    ARGS = p.parse_args()
    os.makedirs(os.path.dirname(ARGS.state), exist_ok=True)
    if not os.path.exists(ARGS.state):
        ecrire_etat({"active_nodes": []})
    serveur = ThreadingHTTPServer(("0.0.0.0", ARGS.port), Handler)
    serveur.serve_forever()


if __name__ == "__main__":
    main()
