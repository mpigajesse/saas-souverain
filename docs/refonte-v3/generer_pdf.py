"""Génère le PDF du journal de refonte V3 à partir de sa source Markdown.

Usage : python docs/refonte-v3/generer_pdf.py [source.md] [sortie.pdf]

Le Markdown reconnu est volontairement restreint : en-tête `---` clé: valeur,
titres `#`/`##`/`###`, paragraphes, listes `-` et `1.`, tableaux `|`,
citations `>`, `**gras**` et `code`. Les titres `#` ouvrent une nouvelle page
et alimentent la table des matières.
"""

import os
import re
import sys
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import cm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    ListFlowable, ListItem, PageBreak, Paragraph, SimpleDocTemplate, Spacer,
    Table, TableStyle,
)
from reportlab.platypus.tableofcontents import TableOfContents

ICI = Path(__file__).resolve().parent
SOURCE = ICI / "AMANE-Refonte-V3.md"
SORTIE = ICI / "AMANE-Refonte-V3.pdf"

BLEU = colors.HexColor("#1F3A5F")
BLEU_CLAIR = colors.HexColor("#E8EEF5")
GRIS = colors.HexColor("#5A6270")
GRIS_TRAIT = colors.HexColor("#C9D1DC")
FOND_CODE = colors.HexColor("#F2F4F7")


def enregistrer_polices():
    """Police TTF avec accents et flèches (→, ≥, ↔) ; Helvetica en dernier recours."""
    candidats = [
        ("C:/Windows/Fonts", "arial.ttf", "arialbd.ttf", "consola.ttf"),
        ("/usr/share/fonts/truetype/dejavu", "DejaVuSans.ttf", "DejaVuSans-Bold.ttf",
         "DejaVuSansMono.ttf"),
        ("/Library/Fonts", "Arial.ttf", "Arial Bold.ttf", "Courier New.ttf"),
    ]
    for dossier, normal, gras, mono in candidats:
        chemins = [os.path.join(dossier, f) for f in (normal, gras, mono)]
        if all(os.path.exists(c) for c in chemins):
            pdfmetrics.registerFont(TTFont("Texte", chemins[0]))
            pdfmetrics.registerFont(TTFont("Texte-Gras", chemins[1]))
            pdfmetrics.registerFont(TTFont("Mono", chemins[2]))
            pdfmetrics.registerFontFamily("Texte", normal="Texte", bold="Texte-Gras")
            return "Texte", "Texte-Gras", "Mono"
    print("Avertissement : aucune police TTF trouvée, repli sur Helvetica "
          "(les flèches et symboles peuvent manquer).")
    return "Helvetica", "Helvetica-Bold", "Courier"


TEXTE, GRAS, MONO = enregistrer_polices()

STYLES = {
    "corps": ParagraphStyle("corps", fontName=TEXTE, fontSize=10, leading=14.5,
                            spaceAfter=6),
    "cellule": ParagraphStyle("cellule", fontName=TEXTE, fontSize=8.5, leading=11.5),
    "entete": ParagraphStyle("entete", fontName=GRAS, fontSize=8.5, leading=11.5,
                             textColor=colors.white),
    "h1": ParagraphStyle("h1", fontName=GRAS, fontSize=18, leading=22,
                         textColor=BLEU, spaceAfter=12),
    "h2": ParagraphStyle("h2", fontName=GRAS, fontSize=13, leading=17,
                         textColor=BLEU, spaceBefore=12, spaceAfter=6),
    "h3": ParagraphStyle("h3", fontName=GRAS, fontSize=11, leading=15,
                         textColor=BLEU, spaceBefore=8, spaceAfter=4),
    "citation": ParagraphStyle("citation", fontName=GRAS, fontSize=10.5, leading=15,
                               textColor=BLEU),
    "toc1": ParagraphStyle("toc1", fontName=GRAS, fontSize=10.5, leading=16,
                           leftIndent=0),
    "toc2": ParagraphStyle("toc2", fontName=TEXTE, fontSize=9.5, leading=14,
                           leftIndent=16, textColor=GRIS),
}


def inline(texte):
    """Convertit le balisage en ligne (code, gras) en balises reportlab."""
    texte = texte.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    morceaux = re.split(r"(`[^`]+`)", texte)
    sortie = []
    for m in morceaux:
        if m.startswith("`") and m.endswith("`") and len(m) > 1:
            sortie.append(f'<font name="{MONO}" color="#8A2D3B">{m[1:-1]}</font>')
        else:
            sortie.append(re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", m))
    return "".join(sortie)


def lire_entete(lignes):
    meta = {}
    if lignes and lignes[0].strip() == "---":
        fin = lignes.index("---", 1)
        for l in lignes[1:fin]:
            cle, _, valeur = l.partition(":")
            meta[cle.strip()] = valeur.strip()
        lignes = lignes[fin + 1:]
    return meta, lignes


def cellules(ligne):
    return [c.strip() for c in ligne.strip().strip("|").split("|")]


def tableau(lignes, largeur):
    entete = cellules(lignes[0])
    corps = [cellules(l) for l in lignes[2:]]
    n = len(entete)
    # Largeurs proportionnelles à la longueur du texte, bornées pour rester lisibles.
    poids = []
    for i in range(n):
        longueurs = [len(r[i]) if i < len(r) else 0 for r in [entete] + corps]
        poids.append(min(max(max(longueurs), 8), 70))
    total = sum(poids)
    largeurs = [largeur * p / total for p in poids]
    donnees = [[Paragraph(inline(c), STYLES["entete"]) for c in entete]]
    for r in corps:
        r = (r + [""] * n)[:n]
        donnees.append([Paragraph(inline(c), STYLES["cellule"]) for c in r])
    t = Table(donnees, colWidths=largeurs, repeatRows=1, hAlign="LEFT")
    t.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, 0), BLEU),
        ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, BLEU_CLAIR]),
        ("GRID", (0, 0), (-1, -1), 0.4, GRIS_TRAIT),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("TOPPADDING", (0, 0), (-1, -1), 4),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
        ("LEFTPADDING", (0, 0), (-1, -1), 5),
        ("RIGHTPADDING", (0, 0), (-1, -1), 5),
    ]))
    return t


def liste(elements, numerotee):
    items = [ListItem(Paragraph(inline(e), STYLES["corps"]), leftIndent=14)
             for e in elements]
    return ListFlowable(items, bulletType="1" if numerotee else "bullet",
                        start="1" if numerotee else "•", leftIndent=14,
                        bulletFontName=TEXTE, bulletFontSize=9,
                        bulletColor=BLEU)


def citation(texte, largeur):
    t = Table([[Paragraph(inline(texte), STYLES["citation"])]], colWidths=[largeur])
    t.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), BLEU_CLAIR),
        ("LINEBEFORE", (0, 0), (0, -1), 3, BLEU),
        ("LEFTPADDING", (0, 0), (-1, -1), 12),
        ("RIGHTPADDING", (0, 0), (-1, -1), 12),
        ("TOPPADDING", (0, 0), (-1, -1), 9),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 9),
    ]))
    return t


class Titre(Paragraph):
    """Paragraphe de titre repéré par la table des matières."""

    def __init__(self, texte, niveau):
        super().__init__(inline(texte), STYLES[f"h{niveau + 1}"])
        self.niveau = niveau
        self.texte_brut = texte


def convertir(lignes, largeur):
    story = []
    i = 0
    premier_h1 = True
    while i < len(lignes):
        l = lignes[i].rstrip()
        if not l.strip():
            i += 1
            continue
        if l.startswith("# "):
            if not premier_h1:
                story.append(PageBreak())
            premier_h1 = False
            story.append(Titre(l[2:], 0))
            i += 1
        elif l.startswith("## "):
            story.append(Titre(l[3:], 1))
            i += 1
        elif l.startswith("### "):
            story.append(Paragraph(inline(l[4:]), STYLES["h3"]))
            i += 1
        elif l.startswith("|"):
            bloc = []
            while i < len(lignes) and lignes[i].startswith("|"):
                bloc.append(lignes[i])
                i += 1
            story.append(tableau(bloc, largeur))
            story.append(Spacer(1, 8))
        elif l.startswith(">"):
            bloc = []
            while i < len(lignes) and lignes[i].startswith(">"):
                bloc.append(lignes[i][1:].strip())
                i += 1
            story.append(citation(" ".join(bloc), largeur))
            story.append(Spacer(1, 8))
        elif re.match(r"^(- |\d+\. )", l):
            numerotee = bool(re.match(r"^\d+\. ", l))
            elements = []
            while i < len(lignes) and re.match(r"^(- |\d+\. )", lignes[i]):
                elements.append(re.sub(r"^(- |\d+\. )", "", lignes[i].strip()))
                i += 1
            story.append(liste(elements, numerotee))
            story.append(Spacer(1, 4))
        else:
            bloc = []
            while (i < len(lignes) and lignes[i].strip()
                   and not re.match(r"^(#|\||>|- |\d+\. )", lignes[i])):
                bloc.append(lignes[i].strip())
                i += 1
            story.append(Paragraph(inline(" ".join(bloc)), STYLES["corps"]))
    return story


def page_de_garde(meta):
    s_titre = ParagraphStyle("gt", fontName=GRAS, fontSize=30, leading=36,
                             textColor=BLEU)
    s_sous = ParagraphStyle("gs", fontName=TEXTE, fontSize=15, leading=21,
                            textColor=GRIS, spaceBefore=10)
    s_doc = ParagraphStyle("gd", fontName=TEXTE, fontSize=11, leading=15,
                           textColor=GRIS, spaceBefore=24)
    infos = [
        ["Version", meta.get("version", "")],
        ["Date", meta.get("date", "")],
        ["Branche", meta.get("branche", "")],
        ["Statut", meta.get("statut", "")],
    ]
    t = Table([[Paragraph(f"<b>{inline(k)}</b>", STYLES["corps"]),
                Paragraph(inline(v), STYLES["corps"])] for k, v in infos],
              colWidths=[3.2 * cm, 11 * cm], hAlign="LEFT")
    t.setStyle(TableStyle([
        ("LINEABOVE", (0, 0), (-1, 0), 1, BLEU),
        ("LINEBELOW", (0, -1), (-1, -1), 1, BLEU),
        ("TOPPADDING", (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
    ]))
    return [
        Spacer(1, 5.5 * cm),
        Paragraph(inline(meta.get("titre", "")), s_titre),
        Paragraph(inline(meta.get("sous-titre", "")), s_sous),
        Paragraph(inline(meta.get("document", "")), s_doc),
        Spacer(1, 2.5 * cm),
        t,
        PageBreak(),
    ]


class Document(SimpleDocTemplate):
    def afterFlowable(self, flowable):
        if isinstance(flowable, Titre):
            self.notify("TOCEntry", (flowable.niveau, flowable.texte_brut, self.page))


def generer(source, sortie):
    lignes = Path(source).read_text(encoding="utf-8").splitlines()
    meta, lignes = lire_entete(lignes)

    doc = Document(str(sortie), pagesize=A4, leftMargin=2 * cm, rightMargin=2 * cm,
                   topMargin=2.2 * cm, bottomMargin=2 * cm,
                   title=meta.get("titre", ""), author="Équipe AMANE",
                   subject=meta.get("sous-titre", ""))
    largeur = doc.width

    def decor(canvas, d):
        canvas.saveState()
        canvas.setStrokeColor(GRIS_TRAIT)
        canvas.setLineWidth(0.5)
        haut = A4[1] - 1.4 * cm
        canvas.line(2 * cm, haut, A4[0] - 2 * cm, haut)
        canvas.setFont(TEXTE, 8)
        canvas.setFillColor(GRIS)
        canvas.drawString(2 * cm, haut + 4, meta.get("titre", ""))
        canvas.drawRightString(A4[0] - 2 * cm, haut + 4,
                               f"Version {meta.get('version', '')} — {meta.get('date', '')}")
        canvas.drawRightString(A4[0] - 2 * cm, 1.2 * cm, f"Page {d.page}")
        canvas.restoreState()

    toc = TableOfContents()
    toc.levelStyles = [STYLES["toc1"], STYLES["toc2"]]
    story = page_de_garde(meta)
    story += [Paragraph("Sommaire", STYLES["h1"]), toc, PageBreak()]
    story += convertir(lignes, largeur)

    doc.multiBuild(story, onFirstPage=lambda c, d: None, onLaterPages=decor)
    print(f"PDF généré : {sortie}")


if __name__ == "__main__":
    src = Path(sys.argv[1]) if len(sys.argv) > 1 else SOURCE
    dst = Path(sys.argv[2]) if len(sys.argv) > 2 else SORTIE
    generer(src, dst)
