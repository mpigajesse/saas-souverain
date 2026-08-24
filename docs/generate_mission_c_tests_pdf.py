#!/usr/bin/env python3
"""Génère docs/mission_c_tests.pdf depuis docs/TESTS_CI.md.

Même pipeline que generate_mission_c_fonctionnement_pdf.py : markdown → HTML
(python-markdown) → PDF (xhtml2pdf/ReportLab), Liberation Mono pour le code.

Usage : .venv/bin/python docs/generate_mission_c_tests_pdf.py
"""
from __future__ import annotations

import pathlib
import re
import sys

import markdown  # type: ignore
from xhtml2pdf import pisa  # type: ignore

ROOT = pathlib.Path(__file__).resolve().parent
MSG_SRC = ROOT / "TESTS_CI.md"
OUT = ROOT / "mission_c_tests.pdf"

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
pre {
  font-family: "MonoMision";
  font-size: 7.2pt;
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
    html = f"<html><head><style>{CSS}</style></head><body>{body}</body></html>"
    with open(OUT, "wb") as f:
        result = pisa.CreatePDF(html, dest=f, encoding="utf-8")
    if result.err:
        print(f"ERREUR: {result.err}", file=sys.stderr)
        return 1
    print(f"PDF généré : {OUT} ({OUT.stat().st_size} octets)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())