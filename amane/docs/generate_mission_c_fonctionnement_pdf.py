#!/usr/bin/env python3
"""Génère docs/mission_c_fonctionnement.pdf depuis docs/MISSION_C_FONCTIONNEMENT.md.

Transforme le markdown de référence en PDF via python-markdown + xhtml2pdf
(ReportLab). Les blocs de code (schémas ASCII) sont rendus en Liberation Mono
(couvre les caractères de dessin de boîtes ┌─│└…).

Usage : .venv/bin/python docs/generate_mission_c_fonctionnement_pdf.py
"""
from __future__ import annotations

import pathlib
import re
import sys

import markdown  # type: ignore
from xhtml2pdf import pisa  # type: ignore

ROOT = pathlib.Path(__file__).resolve().parent
MSG_SRC = ROOT / "MISSION_C_FONCTIONNEMENT.md"
OUT = ROOT / "mission_c_fonctionnement.pdf"

CSS = """
@font-face {
  font-family: "MonoMision";
  src: url("/usr/share/fonts/liberation-mono-fonts/LiberationMono-Regular.ttf");
}
body {
  font-family: Helvetica;
  font-size: 9.5pt;
  line-height: 1.35;
}
h1 { font-size: 17pt; color: #0b3d66; border-bottom: 2pt solid #0b3d66; padding-bottom: 4pt; }
h2 { font-size: 13pt; color: #0b3d66; border-bottom: 1pt solid #bbd;
     padding-bottom: 2pt; margin-top: 16pt; page-break-after: avoid; }
h3 { font-size: 11pt; color: #124; }
img { width: 16cm; margin: 4pt 0; }
pre {
  font-family: "MonoMision";
  font-size: 7.4pt;
  white-space: pre-wrap;
  word-wrap: break-word;
  background-color: #f4f6f8;
  border: 0.5pt solid #ccc;
  padding: 5pt;
  line-height: 1.15;
}
code {
  font-family: "MonoMision";
  font-size: 8.5pt;
  color: #8b1a1a;
}
table {
  border-collapse: collapse;
  width: 100%;
  font-size: 8pt;
  margin: 6pt 0;
}
th, td {
  border: 0.5pt solid #999;
  padding: 3pt 4pt;
  vertical-align: top;
}
th { background-color: #e8eef5; }
blockquote { margin-left: 8pt; color: #444; }
"""


def main() -> int:
    body = markdown.markdown(
        MSG_SRC.read_text(encoding="utf-8"),
        extensions=["tables", "fenced_code", "sane_lists"],
    )
    # xhtml2pdf (v0.2.17) gère mal file:// et basepath : on absolutise les src
    # en chemins fichier bruts (vus par LocalFileURI).
    body = re.sub(
        r'src="[^"]*?([^/"][^"]*\.png)"',
        lambda m: f'src="{ROOT}/{m.group(1)}"',
        body,
    )
    html = f"<html><head><style>{CSS}</style></head><body>{body}</body></html>"
    with open(OUT, "wb") as f:
        result = pisa.CreatePDF(html, dest=f, encoding="utf-8")
    if result.err:
        print(f"ERREUR: {result.err}", file=sys.stderr)
        return 1
    print(f"PDF généré : {OUT} ({OUT.stat().st_size} octets)")
    return 0


if __name__ == "__main__":
    sys.exit(main())