#!/usr/bin/env python3
"""Rejoue la génération de la pile PME (A-07, réplicabilité).

Même gabarit que la pile en service (`templates/tenants/docker-compose.yml.tmpl`),
même tenant et mêmes images épinglées que le banc (`tools/bench/pile-bench.json`).

Deux exercices :
  1. REPRODUCTIBILITÉ : deux rendus avec le MÊME contexte (même secret) et deux
     horodatages distincts → seul l'horodatage d'en-tête doit différer.
  2. SECRETS : deux rendus avec le MÊME horodatage mais deux jetons de relais
     distincts (simulant deux RELAY_SECRET) → seule la ligne RELAY_AUTH_TOKEN
     doit différer.

Sortie : les quatre compose rendus + les deux diffs dans le dossier d'artefacts.
Aucune donnée modifiée, aucune pile touchée.
"""
import datetime
import difflib
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

REPO = pathlib.Path(__file__).resolve().parents[1]
GABARIT = REPO / "templates" / "tenants" / "docker-compose.yml.tmpl"
CONTEXTE = REPO / "tools" / "bench" / "pile-bench.json"
SORTIE = REPO / "docs" / "tests" / "artifacts-a07"

JETON_A = "RELAY_AUTH_TOKEN_REDACTE"
JETON_B = "0" * 64  # autre secret possible : jeton HMAC différent


def rendre(contexte: dict, jeton: str, genere_le: str) -> str:
    ctx = {**contexte, "jeton_relais": jeton, "generated_at": genere_le}
    return Template(GABARIT.read_text(encoding="utf-8")).render(Context(ctx))


def diff(nom: str, a: str, b: str) -> list[str]:
    lignes = list(difflib.unified_diff(
        a.splitlines(), b.splitlines(),
        fromfile=f"{nom}/A", tofile=f"{nom}/B", lineterm=""))
    return lignes


def main() -> int:
    contexte = json.loads(CONTEXTE.read_text(encoding="utf-8"))
    SORTIE.mkdir(parents=True, exist_ok=True)

    # 1. Reproductibilité : même secret, deux horodatages.
    t1 = "25/11/2026 à 21:00"
    t2 = "25/11/2026 à 21:05"
    r1 = rendre(contexte, JETON_A, t1)
    r2 = rendre(contexte, JETON_A, t2)
    (SORTIE / "replay-rendu-1.yml").write_text(r1, encoding="utf-8")
    (SORTIE / "replay-rendu-2.yml").write_text(r2, encoding="utf-8")
    d1 = diff("reproductibilite", r1, r2)
    (SORTIE / "diff-reproductibilite.txt").write_text("\n".join(d1) + "\n", encoding="utf-8")

    # 2. Secrets : même horodatage, deux jetons.
    ra = rendre(contexte, JETON_A, t1)
    rb = rendre(contexte, JETON_B, t1)
    (SORTIE / "secret-A.yml").write_text(ra, encoding="utf-8")
    (SORTIE / "secret-B.yml").write_text(rb, encoding="utf-8")
    d2 = diff("secrets", ra, rb)
    (SORTIE / "diff-secrets.txt").write_text("\n".join(d2) + "\n", encoding="utf-8")

    # Rapport de synthèse.
    lignes = []
    lignes.append("### A-07 — Rejeu de génération de la pile (réplicabilité)")
    lignes.append("")
    lignes.append(f"- Gabarit : `{GABARIT.relative_to(REPO)}`")
    lignes.append(f"- Contexte (même tenant) : `{CONTEXTE.relative_to(REPO)}` "
                  f"— tenant « {contexte['tenant']['name']} » ({contexte['tenant']['id']})")
    lignes.append("")
    lignes.append("#### 1. Reproductibilité (même secret, deux horodatages)")
    lignes.append(f"- Lignes de diff : {len(d1)}")
    lignes.append("```")
    lignes.extend(d1 if d1 else ["(aucune différence)"])
    lignes.append("```")
    lignes.append("")
    lignes.append("#### 2. Secrets (même horodatage, deux jetons de relais)")
    lignes.append(f"- Lignes de diff : {len(d2)}")
    lignes.append("```")
    lignes.extend(d2 if d2 else ["(aucune différence)"])
    lignes.append("```")
    (SORTIE / "synthese-generation.md").write_text("\n".join(lignes) + "\n", encoding="utf-8")

    print("\n".join(lignes))
    return 0


if __name__ == "__main__":
    sys.exit(main())