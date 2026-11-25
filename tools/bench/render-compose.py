#!/usr/bin/env python3
"""Rend templates/tenants/docker-compose.yml.tmpl pour le banc de VM (A-00).

Le banc déploie la pile PME depuis LE gabarit du dépôt, avec le contexte de
la pile de test publiée (tools/bench/pile-bench.json : tenant « AMANE Test »,
images épinglées par digest, application listmonk). Sans argument, le rendu
est écrit sur la sortie standard ; avec --verifier, il est comparé au compose
rendu de la pile en service (/opt/amane-pme/docker-compose.yml) : seuls
l'horodatage d'en-tête doivent différer.

Option --sans-pgadmin : retire le service pgadmin et son volume (le banc ne
charge pas cette image, cf. docs/tests/BANC-VM.md) ; c'est le compose du banc.

Usage : render-compose.py [--sans-pgadmin] [--verifier] [-o sortie.yml]
"""
import argparse
import datetime
import json
import pathlib
import sys

import django
from django.conf import settings

settings.configure(TEMPLATES=[{
    "BACKEND": "django.template.backends.django.DjangoTemplates",
    "DIRS": [],
    "APP_DIRS": False,
    "OPTIONS": {},
}])
django.setup()
from django.template import Context, Template  # noqa: E402

REPO = pathlib.Path(__file__).resolve().parents[2]
GABARIT = REPO / "templates" / "tenants" / "docker-compose.yml.tmpl"
CONTEXTE = REPO / "tools" / "bench" / "pile-bench.json"
REFERENCE = pathlib.Path("/opt/amane-pme/docker-compose.yml")


def retirer_pgadmin(texte: str) -> str:
    """Retire le service pgadmin (et son volume) du compose rendu."""
    lignes = texte.splitlines(keepends=True)
    sortie, dans_pgadmin = [], False
    for ligne in lignes:
        if ligne.startswith("  pgadmin:"):
            dans_pgadmin = True
            sortie.append("# pgadmin retiré pour le banc (A-00) : image non chargée, cf. docs/tests/BANC-VM.md\n")
            continue
        if dans_pgadmin:
            if ligne.startswith("volumes:"):
                dans_pgadmin = False
                sortie.append(ligne)
            continue
        if ligne.rstrip("\n") == "  pgadmin-data:":
            continue
        sortie.append(ligne)
    return "".join(sortie)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--sans-pgadmin", action="store_true")
    p.add_argument("--verifier", action="store_true",
                   help="vérifie le rendu (avec pgadmin) identique à la pile en service")
    p.add_argument("-o", "--sortie", type=pathlib.Path)
    args = p.parse_args()

    contexte = json.loads(CONTEXTE.read_text(encoding="utf-8"))
    rendu = Template(GABARIT.read_text(encoding="utf-8")).render(Context({
        **contexte,
        "generated_at": datetime.datetime.now().strftime("%d/%m/%Y à %H:%M"),
    }))

    if args.verifier:
        reference = REFERENCE.read_text(encoding="utf-8")
        def sans_horodatage(t: str) -> list[str]:
            return [l for l in t.splitlines() if not l.startswith("# Générée le :")]
        if sans_horodatage(rendu) != sans_horodatage(reference):
            for a, b in zip(sans_horodatage(rendu), sans_horodatage(reference)):
                if a != b:
                    print(f"DIFF : {a!r} != {b!r}", file=sys.stderr)
            sys.exit("render-compose : le rendu diffère de la pile en service")
        print("render-compose : rendu identique à la pile en service (horodatage près) OK")

    if args.sans_pgadmin:
        rendu = retirer_pgadmin(rendu)

    if args.sortie:
        args.sortie.write_text(rendu, encoding="utf-8")
    else:
        sys.stdout.write(rendu)
    return 0


if __name__ == "__main__":
    sys.exit(main())
