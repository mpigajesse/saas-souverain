#!/usr/bin/env python3
"""Relit `docs/tests/REGISTRE.md` et vérifie que le registre est cohérent.

Le registre est la source unique des résultats de tests. Un tableau de synthèse
qui n'est pas cohérent avec les lignes est un mensonge : ce script le refuse.

Usage :
    python3 tools/registre.py            # vérifie et affiche la synthèse
    python3 tools/registre.py --ecrire   # réécrit le tableau de synthèse
"""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

REGISTRE = Path(__file__).resolve().parent.parent / "docs" / "tests" / "REGISTRE.md"

# Groupes et effectifs attendus : (libellé, code, prévu, source au plan).
# Un seul endroit fait foi, sinon les deux listes divergent.
GROUPES = [
    ("C — socle cryptographique", "C", 6, "exhaustif §Groupe C"),
    ("N — nœud PME", "N", 6, "exhaustif §Groupe N"),
    ("R — relais zero-knowledge", "R", 3, "exhaustif §Groupe R"),
    ("D — portail éditeur", "D", 6, "exhaustif §Groupe D"),
    ("HA — haute disponibilité", "HA", 6, "amane §HA-01→06"),
    ("E — chaos", "E", 4, "exhaustif §Groupe E"),
    ("M — acceptation E2E", "M", 3, "exhaustif §Groupe M"),
    ("L — charge", "L", 12, "charge §L-01→12"),
    ("A — campagne", "A", 10, "pilot.yaml"),
]

# Résultats admis, dans l'ordre de la synthèse.
STATUTS = ["PASS", "FAIL", "BLOQUE", "NON TENU", "SKIP"]


def lire_lignes(texte: str) -> list[dict]:
    """Extrait les lignes du registre (le tableau sous « ## Registre »)."""
    lignes = []
    dans_registre = False
    for brute in texte.splitlines():
        if brute.startswith("## Registre"):
            dans_registre = True
            continue
        if dans_registre and brute.startswith("## "):
            break
        if not dans_registre or not brute.startswith("|"):
            continue
        cases = [c.strip() for c in brute.strip().strip("|").split("|")]
        if len(cases) < 7:
            continue
        code = cases[0].strip("`")
        if not code or code in ("Code",) or set(code) <= {"-", ":"}:
            continue
        lignes.append({
            "code": code, "intitule": cases[1], "groupe": cases[2],
            "resultat": cases[3], "preuve": cases[4], "date": cases[5],
            "tache": cases[6],
        })
    return lignes


def groupe_de(code: str) -> str:
    m = re.match(r"^(HA|[A-Z]+)-", code)
    return m.group(1) if m else "?"


def synthese(lignes: list[dict]) -> dict[str, dict[str, int]]:
    out = {}
    for libelle, code, prevu, _src in GROUPES:
        out[code] = {s: 0 for s in STATUTS}
        out[code]["total"] = 0
    for ligne in lignes:
        g = ligne["groupe"] if ligne["groupe"] in out else groupe_de(ligne["code"])
        if g not in out:
            out[g] = {s: 0 for s in STATUTS}
            out[g]["total"] = 0
        out[g]["total"] += 1
        statut = ligne["resultat"].upper().replace(" ", " ")
        if statut in STATUTS:
            out[g][statut] += 1
    return out


def tableau(syn: dict[str, dict[str, int]]) -> str:
    entetes = ("| Groupe | Plan | Prévus | Passés | Échecs | Bloqués | Non tenus | Ignorés |")
    sep = "|---|---|---:|---:|---:|---:|---:|---:|"
    lignes = [entetes, sep]
    for libelle, code, prevu, _src in GROUPES:
        s = syn.get(code, {})
        lignes.append(
            f"| {libelle} | {code} | {prevu} | {s.get('PASS',0)} | {s.get('FAIL',0)} "
            f"| {s.get('BLOQUE',0)} | {s.get('NON TENU',0)} | {s.get('SKIP',0)} |")
    return "\n".join(lignes)


def main() -> int:
    if not REGISTRE.exists():
        print(f"registre absent : {REGISTRE}", file=sys.stderr)
        return 2
    texte = REGISTRE.read_text(encoding="utf-8")
    lignes = lire_lignes(texte)
    syn = synthese(lignes)
    attendu = tableau(syn)

    # Le tableau de synthèse du fichier doit être identique à celui calculé.
    bloc = re.search(r"## Synthèse de couverture\n\n(.*?)\n\n## ", texte, re.S)
    probleme = False
    if not bloc:
        print("tableau de synthèse introuvable", file=sys.stderr)
        probleme = True
    else:
        actuel = bloc.group(1).strip()
        if "--ecrire" in sys.argv:
            texte = texte.replace(actuel, attendu, 1)
            REGISTRE.write_text(texte, encoding="utf-8")
            print("synthèse réécrite")
            return 0
        if actuel != attendu.strip():
            probleme = True
            print("SYNTHÈSE INCOHÉRENTE — le tableau ne reflète pas les lignes :")
            print("--- attendu ---")
            print(attendu)
            print("--- trouvé ---")
            print(actuel)

    # Contrôles de fond : un résultat doit être un statut admis, une preuve
    # doit exister, un code doit être unique.
    codes: dict[str, int] = defaultdict(int)
    for ligne in lignes:
        codes[ligne["code"]] += 1
        if ligne["resultat"].upper() not in STATUTS:
            print(f"statut inconnu : {ligne['code']} -> {ligne['resultat']!r}")
            probleme = True
        if ligne["resultat"].upper() == "PASS" and not ligne["preuve"]:
            print(f"PASS sans preuve : {ligne['code']}")
            probleme = True
        if ligne["preuve"] and not ligne["preuve"].startswith(("docs/", "load/", "tools/")):
            print(f"preuve hors dépôt : {ligne['code']} -> {ligne['preuve']}")
            probleme = True
    doublons = {c: n for c, n in codes.items() if n > 1}
    if doublons:
        print(f"codes en double : {doublons}")
        probleme = True

    print(f"{len(lignes)} ligne(s) de registre, {len(codes)} code(s) distinct(s)")
    print(attendu)
    return 1 if probleme else 0


if __name__ == "__main__":
    sys.exit(main())