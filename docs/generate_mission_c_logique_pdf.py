#!/usr/bin/env python
"""Génère docs/mission_c_logique.pdf — « La logique globale de Mission C ».

Document de compréhension (pas un manuel ligne à ligne) : ce que fait le
système, comment il le fait, et quelle logique guide chaque mécanisme.
Tous les schémas sont dessinés en vectoriel avec ReportLab (graphics/shapes),
donc le PDF est autonome : aucune image externe requise.

Lancement (depuis la racine du repo) :
    .venv/bin/python docs/generate_mission_c_logique_pdf.py
Sortie :
    docs/mission_c_logique.pdf
"""

from __future__ import annotations

import math
import re
import sys
from datetime import date
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib.units import mm
from reportlab.graphics.shapes import (
    Drawing,
    Line,
    Polygon,
    Rect,
    String,
)
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    NextPageTemplate,
    PageBreak,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)

REPO = Path(__file__).resolve().parent.parent
PROTO_FILE = REPO / "proto" / "amane" / "framework" / "v1" / "framework.proto"
OUT = Path(__file__).resolve().parent / "mission_c_logique.pdf"

# ---------------------------------------------------------------------------
# Polices : Carlito si disponible (Unicode complet : → ≤ ≈ …), sinon Helvetica.
# ---------------------------------------------------------------------------
_FONTS_DIR = Path("/usr/share/fonts/google-carlito-fonts")


def _register_fonts() -> dict:
    names = {
        "reg": "Carlito-Regular.ttf",
        "bold": "Carlito-Bold.ttf",
        "ital": "Carlito-Italic.ttf",
        "bi": "Carlito-BoldItalic.ttf",
    }
    try:
        for key, fname in names.items():
            pdfmetrics.registerFont(TTFont("Carlito" if key == "reg" else
                                           {"bold": "Carlito-Bold",
                                            "ital": "Carlito-Italic",
                                            "bi": "Carlito-BoldItalic"}[key],
                                           str(_FONTS_DIR / fname)))
        pdfmetrics.registerFontFamily(
            "Carlito", normal="Carlito", bold="Carlito-Bold",
            italic="Carlito-Italic", boldItalic="Carlito-BoldItalic")
        return {"base": "Carlito", "bold": "Carlito-Bold",
                "ital": "Carlito-Italic", "mono": "Courier",
                "mono_bold": "Courier-Bold"}
    except Exception:
        return {"base": "Helvetica", "bold": "Helvetica-Bold",
                "ital": "Helvetica-Oblique", "mono": "Courier",
                "mono_bold": "Courier-Bold"}


F = _register_fonts()

# ---------------------------------------------------------------------------
# Palette
# ---------------------------------------------------------------------------
ACCENT = colors.HexColor("#1F3A5F")     # bleu nuit AMANE
BLUE = colors.HexColor("#2C5F8A")
BLUE_BG = colors.HexColor("#E3EDF6")
LIGHT = colors.HexColor("#EEF2F7")
MID = colors.HexColor("#C9D4E1")
GREEN = colors.HexColor("#2E7D32")
GREEN_BG = colors.HexColor("#E3F1E4")
RED = colors.HexColor("#B3352C")
RED_BG = colors.HexColor("#FAE7E5")
AMBER = colors.HexColor("#9A5B00")
AMBER_BG = colors.HexColor("#FDF1DC")
GREY = colors.HexColor("#5B6570")
GREY_BG = colors.HexColor("#EFF1F3")
WHITE = colors.white
INK = colors.HexColor("#22272E")

PAGE_W, PAGE_H = A4
MARGIN = 17 * mm
USABLE = PAGE_W - 2 * MARGIN          # ≈ 493 pt


def esc(s: str) -> str:
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


# ---------------------------------------------------------------------------
# Styles de paragraphes
# ---------------------------------------------------------------------------
def _ps(name, **kw):
    base = dict(fontName=F["base"], fontSize=9.5, leading=13.5,
                textColor=INK, spaceAfter=5)
    base.update(kw)
    return ParagraphStyle(name, **base)


st_title = _ps("title", fontSize=27, leading=32, textColor=ACCENT,
               fontName=F["bold"])
st_subtitle = _ps("subtitle", fontSize=13, leading=18, textColor=GREY)
st_h1 = _ps("h1", fontSize=15, leading=19, textColor=ACCENT,
            fontName=F["bold"], spaceBefore=16, spaceAfter=5)
st_h2 = _ps("h2", fontSize=11.5, leading=15, textColor=BLUE,
            fontName=F["bold"], spaceBefore=9, spaceAfter=3)
st_body = _ps("body")
st_lead = _ps("lead", fontSize=10.5, leading=15.5, textColor=INK)
st_small = _ps("small", fontSize=8, leading=11, textColor=GREY)
st_caption = _ps("caption", fontSize=8, leading=11, textColor=GREY,
                 spaceBefore=2)
st_bullet = _ps("bullet", leftIndent=10, bulletIndent=0, spaceAfter=3)
st_code = _ps("code", fontName=F["mono"], fontSize=8, leading=11.2,
              textColor=INK, backColor=colors.HexColor("#F4F6F8"),
              borderPadding=5, spaceBefore=3, spaceAfter=7)


def bullets(items: list[str]) -> list[Paragraph]:
    return [Paragraph(f"<font color='#2C5F8A'>▪</font>&nbsp;&nbsp;{it}", st_bullet)
            for it in items]


_cell_cache: dict = {}


def table(rows, widths, header=True, fs=8, align_left=True):
    def cell_style(bold=False, color=INK):
        key = (fs, bold, color)
        if key not in _cell_cache:
            _cell_cache[key] = ParagraphStyle(
                f"cell{fs}{bold}{color}", fontName=F["bold"] if bold else F["base"],
                fontSize=fs, leading=fs + 2.8, textColor=color)
        return _cell_cache[key]

    data = []
    for ri, row in enumerate(rows):
        out = []
        for c in row:
            if isinstance(c, str):
                hd = header and ri == 0
                out.append(Paragraph(c, cell_style(bold=hd,
                                                   color=WHITE if hd else INK)))
            else:
                out.append(c)
        data.append(out)
    t = Table(data, colWidths=widths, repeatRows=1 if header else 0)
    style = [
        ("GRID", (0, 0), (-1, -1), 0.4, MID),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("TOPPADDING", (0, 0), (-1, -1), 3.5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3.5),
        ("LEFTPADDING", (0, 0), (-1, -1), 5),
        ("RIGHTPADDING", (0, 0), (-1, -1), 5),
    ]
    if header:
        style += [
            ("BACKGROUND", (0, 0), (-1, 0), ACCENT),
        ]
        for r in range(2, len(rows), 2):
            style.append(("BACKGROUND", (0, r), (-1, r), LIGHT))
    t.setStyle(TableStyle(style))
    return t


def mono(s: str, size=7.6) -> str:
    return f"<font face='{F['mono']}' size='{size}'>{esc(s)}</font>"


# ===========================================================================
# Mini-DSL de schémas vectoriels
# ===========================================================================
class Dia:
    """Wrapper autour de Drawing avec des primitives métier."""

    def __init__(self, w: float, h: float):
        self.w, self.h = w, h
        self.d = Drawing(w, h)

    def add(self):
        self.d.hAlign = "CENTER"
        return self.d

    # --- texte ------------------------------------------------------------
    def txt(self, x, y, s, size=8, color=INK, bold=False, anchor="start",
            italic=False, font=None):
        self.d.add(String(x, y, s, fontName=font or (F["bold"] if bold else
                          (F["ital"] if italic else F["base"])),
                          fontSize=size, fillColor=color, textAnchor=anchor))

    def tw(self, s, size=8, bold=False):
        return pdfmetrics.stringWidth(s, F["bold"] if bold else F["base"], size)

    def label(self, cx, cy, s, size=7, color=INK, bold=False, bg=None,
              pad=2.5, anchor="middle", italic=False):
        w = self.tw(s, size, bold)
        x0 = {"middle": cx - w / 2, "start": cx, "end": cx - w}[anchor]
        if bg is not None:
            self.d.add(Rect(x0 - pad, cy - size * 0.36, w + 2 * pad,
                            size * 1.28, fillColor=bg, strokeColor=None))
        self.txt(x0, cy - size * 0.36 + 1.2, s, size=size, color=color,
                 bold=bold, italic=italic, anchor="start")

    # --- formes -----------------------------------------------------------
    def box(self, x, y, w, h, lines, fill=LIGHT, stroke=ACCENT, sw=1.0,
            fs=8, tcolor=INK, radius=4, bold_first=True, lead=None,
            dash=None, valign="middle"):
        self.d.add(Rect(x, y, w, h, rx=radius, ry=radius, fillColor=fill,
                        strokeColor=stroke, strokeWidth=sw,
                        strokeDashArray=dash))
        lead = lead or fs + 2.6
        n = len(lines)
        total = n * lead - (lead - fs)
        cy = (y + h / 2) if valign == "middle" else (y + h - fs - 4)
        ty = cy + total / 2 - fs + (0 if n > 1 else 0)
        for i, ln in enumerate(lines):
            bold = bold_first and i == 0 and len(lines) > 1
            col = tcolor if not (bold and fill in (ACCENT,)) else WHITE
            self.txt(x + w / 2, ty - i * lead, ln,
                     size=fs if not bold else fs + 0.4,
                     color=col, bold=bold, anchor="middle")

    def chip(self, cx, cy, s, fill, stroke, tcolor=INK, fs=7.2, pad=5,
             h=None, bold=False):
        w = self.tw(s, fs, bold) + 2 * pad
        hh = h or fs + 6.5
        self.d.add(Rect(cx - w / 2, cy - hh / 2, w, hh, rx=hh / 2, ry=hh / 2,
                        fillColor=fill, strokeColor=stroke, strokeWidth=0.9))
        self.txt(cx, cy - fs * 0.36, s, size=fs, color=tcolor, bold=bold,
                 anchor="middle")
        return w

    def diamond(self, cx, cy, w, h, lines, fill=AMBER_BG, stroke=AMBER,
                fs=7.4):
        self.d.add(Polygon(points=[cx, cy + h / 2, cx + w / 2, cy, cx,
                                   cy - h / 2, cx - w / 2, cy],
                           fillColor=fill, strokeColor=stroke, strokeWidth=1))
        lead = fs + 2.4
        ty = cy + (len(lines) - 1) * lead / 2 - fs * 0.36
        for i, ln in enumerate(lines):
            self.txt(cx, ty - i * lead, ln, size=fs, anchor="middle")

    # --- connecteurs ------------------------------------------------------
    def arrow(self, pts, color=BLUE, sw=1.3, dash=None, head=6.0,
              double=False):
        pts = [(float(a), float(b)) for a, b in pts]
        arr = []
        for i in range(len(pts) - 1):
            (x1, y1), (x2, y2) = pts[i], pts[i + 1]
            self.d.add(Line(x1, y1, x2, y2, strokeColor=color, strokeWidth=sw,
                            strokeDashArray=dash, strokeLineCap=1))
        def head_at(p_from, p_to):
            ang = math.atan2(p_to[1] - p_from[1], p_to[0] - p_from[0])
            b = math.pi / 9
            p1 = (p_to[0] - head * math.cos(ang - b),
                  p_to[1] - head * math.sin(ang - b))
            p2 = (p_to[0] - head * math.cos(ang + b),
                  p_to[1] - head * math.sin(ang + b))
            self.d.add(Polygon(points=[p_to[0], p_to[1], p1[0], p1[1],
                                       p2[0], p2[1]],
                               fillColor=color, strokeColor=color,
                               strokeWidth=0.4))
        head_at(pts[-2], pts[-1])
        if double:
            head_at(pts[1], pts[0])

    def elbow(self, x1, y1, x2, y2, mid_y, **kw):
        self.arrow([(x1, y1), (x1, mid_y), (x2, mid_y), (x2, y2)], **kw)


# ===========================================================================
# Schéma 1 — Architecture globale
# ===========================================================================
def diag_architecture() -> Drawing:
    D = Dia(USABLE, 336)
    H = D.h

    # Clients
    D.box(146, H - 34, 200, 26,
          ["Missions A & B — clients (SDK)"], fill=GREY_BG, stroke=GREY,
          fs=8.6, bold_first=False)
    D.label(246, H - 42, "gRPC sur TLS 1.3 · mTLS (certificat client exigé)",
            size=7.4, color=GREY, bg=WHITE)

    # Conteneurs sites
    top_y, cont_h = 96, 176
    for x0, name, sub in ((10, "SITE A", "ex. atelier"),
                          (USABLE - 235, "SITE B", "ex. e-shop")):
        D.d.add(Rect(x0, top_y, 225, cont_h, rx=7, ry=7,
                     fillColor=colors.HexColor("#F7FAFC"), strokeColor=MID,
                     strokeWidth=1.1))
        D.label(x0 + 112, top_y + cont_h - 11, f"{name}",
                size=8.4, color=ACCENT, bold=True)
        D.label(x0 + 112, top_y + cont_h - 21.5, sub, size=6.6, color=GREY,
                italic=False)

    ax_a, ax_b = 122, USABLE - 123          # centres X des sites
    # Boîte orchestrator
    oy, oh = top_y + 78, 40
    for cx in (ax_a, ax_b):
        D.box(cx - 102, oy, 204, oh,
              ["Orchestrator Go", "serveur gRPC · fencing · relay CRDT",
               "journal · superviseur"],
              fill=BLUE_BG, stroke=BLUE, fs=7.4)
    # flèches clients -> orchestrators
    D.elbow(196, H - 34, ax_a, oy + oh, H - 46)
    D.elbow(296, H - 34, ax_b, oy + oh, H - 46)

    # etcd + postgres
    iy, ih = top_y + 26, 34
    for cx in (ax_a, ax_b):
        D.box(cx - 102, iy, 98, ih,
              ["etcd (Raft)", "lease · membres · heartbeats"],
              fill=LIGHT, stroke=ACCENT, fs=6.8)
        D.box(cx + 4, iy, 98, ih,
              ["PostgreSQL + Patroni", "primary / réplica synchrone"],
              fill=LIGHT, stroke=ACCENT, fs=6.8)
        D.arrow([(cx - 53, oy), (cx - 53, iy + ih)], color=GREY, sw=1.0)
        D.label(cx - 53, oy + 8, "lock / élection", size=6, color=GREY)
        D.arrow([(cx + 53, iy + ih), (cx + 53, oy)], color=GREY, sw=1.0)
        D.label(cx + 53, oy + 8, "écritures gated", size=6, color=GREY)

    # bande wireguard
    wy, wh = top_y + 4, 16
    for cx in (ax_a, ax_b):
        D.chip(cx, wy + wh / 2, "mesh WireGuard intra-site (tunnels /32)",
               BLUE_BG, BLUE, tcolor=BLUE, fs=6.6)

    # lien intersite PushDelta (sous les sites)
    mid_y = 74
    D.elbow(ax_a, top_y, ax_b, top_y, mid_y, color=GREEN, sw=1.6, double=True)
    D.label((ax_a + ax_b) / 2, mid_y + 9,
            "PushDelta — deltas CRDT de stock (TLS sortant)",
            size=7.6, color=GREEN, bold=True)
    D.label((ax_a + ax_b) / 2, mid_y - 13,
            "asynchrone : tolère latence et coupures (mode AP)",
            size=6.8, color=GREY)

    # légende
    ly = 18
    lx = 60
    D.chip(lx, ly, "contrôle (consensus)", LIGHT, ACCENT, fs=6.8)
    D.chip(lx + 130, ly, "données (journal, stock)", BLUE_BG, BLUE, fs=6.8)
    D.chip(lx + 288, ly, "réseau chiffré", GREEN_BG, GREEN, fs=6.8)
    return D.add()


# ===========================================================================
# Schéma 2 — Cycle de vie leadership / fencing
# ===========================================================================
def diag_fencing() -> Drawing:
    D = Dia(USABLE, 218)
    H = D.h
    cy = H - 84
    bw, bh = 132, 64
    xs = [14, (USABLE - bw) / 2, USABLE - bw - 14]

    D.box(xs[0], cy - bh / 2, bw, bh,
          ["1. CAMPAGNE", "session etcd : lease TTL 30 s",
           "Election.Campaign(nodeID)"], fill=LIGHT, stroke=ACCENT, fs=7.4)
    D.box(xs[1], cy - bh / 2, bw, bh,
          ["2. LEADER", "keepalive continu de la lease",
           "/amane/leader = nodeID", "écritures acceptées"],
          fill=GREEN_BG, stroke=GREEN, fs=7.4)
    D.box(xs[2], cy - bh / 2, bw, bh,
          ["3. FENCING", "lease expirée ou perdue",
           "IsLeader() = false", "Write → FailedPrecondition"],
          fill=RED_BG, stroke=RED, fs=7.4)

    ya, yb = cy + bh / 2, cy - bh / 2
    D.arrow([(xs[0] + bw, cy), (xs[1], cy)])
    D.label((xs[0] + bw + xs[1]) / 2, ya + 8, "élection : un seul gagnant",
            size=6.8, color=BLUE, bg=WHITE)
    D.arrow([(xs[1] + bw, cy), (xs[2], cy)], color=RED)
    D.label((xs[1] + bw + xs[2]) / 2, ya + 8, "crash · coupure réseau",
            size=6.8, color=RED, bg=WHITE)
    D.elbow(xs[2] + bw / 2, yb, xs[0] + bw / 2, yb, yb - 34, color=BLUE,
            dash=[4, 3])
    D.label(USABLE / 2, yb - 40,
            "boucle Run() : re-campagne automatique dès que possible",
            size=6.8, color=BLUE, italic=True)

    # socle etcd
    D.box(USABLE / 2 - 80, 16, 160, 24,
          ["etcd — Raft (source de vérité distribuée)"],
          fill=GREY_BG, stroke=GREY, fs=7.2, bold_first=False)
    for x in xs:
        D.d.add(Line(x + bw / 2, 40, x + bw / 2, ya - bh / 2 - 2,
                     strokeColor=MID, strokeWidth=0.8, strokeDashArray=[2, 2]))
    D.label(USABLE / 2, 8,
            "arrêt gracieux : session.Close() → la clé d'élection est supprimée, "
            "le fencing est immédiat", size=6.8, color=GREY)
    return D.add()


# ===========================================================================
# Schéma 3 — Chemin d'écriture gated
# ===========================================================================
def diag_write_path() -> Drawing:
    D = Dia(USABLE, 262)
    H = D.h
    cy = H - 120

    D.box(10, cy - 24, 88, 48,
          ["Client B", "Write(journal,", "payload chiffré)"],
          fill=GREY_BG, stroke=GREY, fs=7.4)
    dx = 196
    D.diamond(dx, cy, 128, 74,
              ["Nœud leader ?", "garde IsLeader()", "avant tout accès journal"])

    # branche refusée (haut)
    rb_y = H - 52
    D.box(330, rb_y, 156, 40,
          ["REFUS", "codes.FailedPrecondition", "« nœud non leader (fencing) »"],
          fill=RED_BG, stroke=RED, fs=7.2)
    D.arrow([(dx + 40, cy + 26), (330, rb_y + 20)], color=RED)
    D.label(268, cy + 40, "non", size=7.4, color=RED, bold=True)

    # branche acceptée (bas)
    jb_y = 96
    D.box(300, jb_y, 150, 52,
          ["Journal.Append", "(journal_id, op_seq, payload)",
           "idempotent : un rejeu rend", "le même committed_seq"],
          fill=LIGHT, stroke=BLUE, fs=7.2)
    D.arrow([(dx + 46, cy - 26), (300, jb_y + 26)], color=GREEN)
    D.label(262, cy - 40, "oui", size=7.4, color=GREEN, bold=True)

    ob_y = 22
    D.box(300, ob_y, 150, 30,
          ["OK — committed_seq monotone", "réplication synchrone confirmée"],
          fill=GREEN_BG, stroke=GREEN, fs=7.2)
    D.arrow([(375, jb_y), (375, ob_y + 30)], color=GREEN)

    # bandeau notes
    D.d.add(Rect(10, 22, 268, 56, rx=5, ry=5, fillColor=AMBER_BG,
                 strokeColor=AMBER, strokeWidth=0.8))
    D.txt(18, 66, "À retenir", size=7.4, color=AMBER, bold=True)
    for i, ln in enumerate([
        "• Seul le leader écrit : jamais deux séquences concurrentes.",
        "• Ping, Read, Enroll, NotifyRevocation, PushDelta : non gated.",
        "• Le clair n'existe pas côté C : payload chiffré par la Mission A.",
    ]):
        D.txt(18, 54 - i * 11, ln, size=7.0, color=INK)

    D.arrow([(98, cy), (dx - 64, cy)], color=BLUE)
    return D.add()


# ===========================================================================
# Schéma 4 — Convergence CRDT (exemple chiffré)
# ===========================================================================
def diag_crdt() -> Drawing:
    D = Dia(USABLE, 218)
    H = D.h

    D.label(USABLE / 2, H - 10,
            "Scénario : stock partagé. e-shop : +5 puis +7 — atelier : −3. "
            "Attendu partout : 4.", size=8.2, color=INK, bold=True)

    py, ph = 100, 92
    pw = 216
    xa, xb = 12, USABLE - pw - 12
    for x0, name, ops in (
        (xa, "NŒUD e-shop", ["Add(+5)   → Δ{inc: 5, seq 1}",
                             "Add(+7)   → Δ{inc: 7, seq 2}   (total cumulé)"]),
        (xb, "NŒUD atelier", ["Add(−3)   → Δ{dec: 3, seq 1}",
                              "                              "]),
    ):
        D.box(x0, py, pw, ph, [name], fill=LIGHT, stroke=ACCENT, fs=8,
              valign="top")
        yy = py + ph - 26
        for op in ops:
            D.d.add(Rect(x0 + 12, yy - 9, pw - 24, 15, rx=3, ry=3,
                         fillColor=WHITE, strokeColor=MID, strokeWidth=0.7))
            D.txt(x0 + 18, yy - 4.6, op, size=7.0,
                  font=F["mono"])
            yy -= 20
        D.chip(x0 + pw / 2, py + 14,
               "fusion : Inc{e-shop:7} Dec{atelier:3} → valeur 4",
               GREEN_BG, GREEN, tcolor=GREEN, fs=7.0, bold=True)

    # échanges
    my = py + ph / 2
    D.arrow([(xa + pw, my + 14), (xb, my + 14)], color=GREEN, double=True)
    D.arrow([(xb, my - 16), (xa + pw, my - 16)], color=GREEN, double=True)
    D.label(USABLE / 2, my + 24, "PushDelta", size=7.2, color=GREEN,
            bold=True, bg=WHITE)
    D.label(USABLE / 2, py - 12,
            "doublons · réordonnancement · trous de seq : SANS effet",
            size=7.0, color=GREY, bg=WHITE)

    D.label(USABLE / 2, 52,
            "Un delta porte le TOTAL cumulé de l'émetteur (pas un incrément "
            "relatif). La fusion « max par nœud émetteur » est alors",
            size=7.6, color=INK)
    D.label(USABLE / 2, 39,
            "commutative · associative · idempotente  ⇒  convergence garantie "
            "(propriété AP des CRDT state-based)",
            size=7.6, color=GREEN, bold=True)
    return D.add()


# ===========================================================================
# Schéma 5 — Boucle de propagation (Propagator)
# ===========================================================================
def diag_propagator() -> Drawing:
    D = Dia(USABLE, 128)
    H = D.h
    cy = H - 46
    steps = [
        ("Relay", "Add() met à jour\nle pending (seq++)", LIGHT, ACCENT),
        ("Outgoing()", "deltas non confirmés\ntirés à chaque tick", LIGHT, ACCENT),
        ("Propagator", "tick 1 s (défaut)\npush vers le pair", BLUE_BG, BLUE),
        ("Pair : Accept()", "fusion max-par-nœud\n(réflexion ignorée)", BLUE_BG, BLUE),
        ("ack → Confirm()", "gc du pending\n(fuite mémoire évitée)", GREEN_BG, GREEN),
    ]
    bw, bh, gap = 88, 44, None
    gap = (USABLE - 20 - len(steps) * bw) / (len(steps) - 1)
    centers = []
    x = 10
    for title, sub, fill, stroke in steps:
        D.box(x, cy - bh / 2, bw, bh, [], fill=fill, stroke=stroke, fs=7.4)
        D.txt(x + bw / 2, cy + bh / 2 - 11, title, size=7.4, bold=True,
              color=stroke if stroke != MID else INK, anchor="middle")
        yy = cy + bh / 2 - 23
        for ln in sub.split("\n"):
            D.txt(x + bw / 2, yy, ln, size=6.2, color=INK, anchor="middle")
            yy -= 8.6
        centers.append(x + bw / 2)
        x += bw + gap
    for i in range(len(steps) - 1):
        x1 = centers[i] + bw / 2
        x2 = centers[i + 1] - bw / 2
        D.arrow([(x1, cy), (x2, cy)], color=BLUE, sw=1.2)
    # boucle retour
    D.elbow(centers[-1], cy - bh / 2, centers[0], cy - bh / 2, 14,
            color=GREEN, dash=[4, 3])
    D.label(USABLE / 2, 6,
            "échec réseau : le push est retenté au tick suivant — les deltas "
            "restent dans le pending, rien n'est jamais perdu",
            size=7.0, color=RED)
    return D.add()


# ===========================================================================
# Schéma 6 — Arbre de décision du superviseur
# ===========================================================================
def diag_supervisor_flow() -> Drawing:
    D = Dia(USABLE, 470)
    cx = 158
    tx = 386  # colonne des terminaisons

    def vlink(y_from, y_to, cond="", cond_col=RED):
        D.arrow([(cx, y_from), (cx, y_to)], color=cond_col)
        if cond:
            D.label(cx + 9, (y_from + y_to) / 2 - 2.4, cond, size=6.8,
                    color=cond_col, bold=True, bg=WHITE)

    # 1. tick
    D.chip(cx, 458, "tick — toutes les 500 ms", GREY_BG, GREY, fs=7.4,
           bold=True)
    # 2. sonde + heartbeat
    D.box(cx - 118, 398, 236, 34,
          ["sonde GET /patroni de chaque nœud (timeout 300 ms)",
           "lit le lock Patroni + publie le heartbeat LOCAL (lease 2 s)"],
          fill=LIGHT, stroke=ACCENT, fs=6.9)
    vlink(451, 432, "", BLUE)
    # 3. leader joignable ?
    D.diamond(cx, 366, 152, 54, ["leader REST", "joignable ?"])
    vlink(398, 393, "", BLUE)
    D.arrow([(cx + 76, 366), (tx - 90, 366)], color=GREEN)
    D.label((cx + 76 + tx - 90) / 2, 373, "oui", size=6.8, color=GREEN,
            bold=True, bg=WHITE)
    D.box(tx - 90, 352, 172, 28,
          ["sain — rien à faire (compteur remis à zéro)"],
          fill=GREEN_BG, stroke=GREEN, fs=6.9, bold_first=False)
    # 4. heartbeat frais ?
    D.diamond(cx, 292, 172, 56, ["heartbeat etcd du leader", "FRAIS (role=primary) ?"])
    vlink(339, 320, "non", RED)
    D.arrow([(cx + 86, 292), (tx - 90, 292)], color=AMBER)
    D.label((cx + 86 + tx - 90) / 2, 299, "oui", size=6.8, color=AMBER,
            bold=True, bg=WHITE)
    D.box(tx - 90, 274, 172, 36,
          ["GARDE ANTI-PARTITION", "primary vivant mais isolé : JAMAIS de",
           "forçage — seule Patroni tranche"],
          fill=AMBER_BG, stroke=AMBER, fs=6.9)
    # 5. suspicion
    D.box(cx - 118, 234, 236, 26,
          ["suspicion : ticks consécutifs +1  (StaleConfirm = 2)"],
          fill=LIGHT, stroke=ACCENT, fs=6.9)
    vlink(264, 260, "non", RED)
    # 6. assez de ticks ?
    D.diamond(cx, 186, 140, 46, ["≥ 2 ticks", "consécutifs ?"])
    vlink(234, 209, "", RED)
    D.arrow([(cx + 70, 186), (tx - 90, 186)], color=GREY)
    D.label((cx + 70 + tx - 90) / 2, 193, "non", size=6.8, color=GREY,
            bold=True, bg=WHITE)
    D.box(tx - 90, 172, 172, 28,
          ["attendre le tick suivant (anti-faux positif)"],
          fill=GREY_BG, stroke=GREY, fs=6.9, bold_first=False)
    # 7. droit de forcer
    D.box(cx - 118, 128, 236, 26,
          ["lock etcd « droit de forcer » : une seule instance agit (pas de SPOF)"],
          fill=LIGHT, stroke=ACCENT, fs=6.9)
    vlink(163, 154, "oui", GREEN)
    # 8. libération lock Patroni
    D.box(cx - 118, 84, 236, 30,
          ["DELETE du lock Patroni /service/<scope>/leader",
           "sinon la promotion attendrait la lease ttl ≥ 20 s"],
          fill=LIGHT, stroke=AMBER, fs=6.9)
    vlink(128, 114, "", GREEN)
    # 9. POST /failover
    D.box(cx - 118, 40, 236, 26,
          ["POST /failover {candidate : réplica sync, force} — sans champ leader"],
          fill=GREEN_BG, stroke=GREEN, fs=6.9)
    vlink(84, 66, "", GREEN)
    # 10. fin
    D.chip(cx, 16, "Patroni promeut — zéro perte (synchrone) · cooldown 30 s",
           GREEN_BG, GREEN, tcolor=GREEN, fs=7.2, bold=True)
    vlink(40, 24, "", GREEN)
    return D.add()


# ===========================================================================
# Schéma 7 — Frise des mesures de bascule
# ===========================================================================
def diag_timeline() -> Drawing:
    D = Dia(USABLE, 216)
    H = D.h
    x0, x1 = 128, USABLE - 24
    tmax = 22.0

    def X(t):
        return x0 + (x1 - x0) * t / tmax

    rows = [
        ("crash SIGKILL\n+ superviseur", [
            (0.0, 0.8, BLUE, "détection 0,8 s"),
            (0.8, 3.2, GREEN, "POST /failover + promotion"),
        ], 4.0, "writable 4,0 s", "above"),
        ("partition réseau\n+ superviseur", [
            (0.0, 0.7, BLUE, ""),
            (0.7, 3.0, GREEN, ""),
        ], 3.4, "writable 3,4 s", "below"),
        ("switchover\nplanifié", [
            (0.0, 2.2, colors.HexColor("#00796B"), "bascule contrôlée"),
        ], 2.4, "writable ~2,4 s", "above"),
        ("crash SANS\nsuperviseur", [
            (0.0, 21.0, RED, "attente de l'expiration de la lease Patroni (ttl ≥ 20 s)"),
        ], None, None, "above"),
    ]

    top = H - 34
    rh, rgap = 17, 22
    for i, (name, segs, mark, mark_lab, mark_pos) in enumerate(rows):
        ry = top - i * (rh + rgap)
        cy = ry - rh / 2
        for j, ln in enumerate(name.split("\n")):
            D.txt(x0 - 8, cy + (0 if j == 0 else -9), ln, size=7.0,
                  color=INK, anchor="end", bold=(j == 0))
        D.d.add(Rect(X(0), ry - rh, X(tmax) - X(0), rh, fillColor=colors
                     .HexColor("#F6F8FA"), strokeColor=None))
        below_slot = 0
        for (ta, tb, col, lab) in segs:
            D.d.add(Rect(X(ta), ry - rh + 2, max(X(tb) - X(ta), 2), rh - 4,
                         rx=3, ry=3, fillColor=col, strokeColor=None))
            if not lab:
                continue
            lab_w = D.tw(lab, 6.4, True)
            if tb - ta > 2.5 and (X(tb) - X(ta)) > lab_w + 10:
                D.label((X(ta) + X(tb)) / 2, cy - 2.4, lab, size=6.4,
                        color=WHITE, bold=True)
            else:
                # segment étroit : libellé sous la barre, dans la couleur,
                # empilé s'il y en a plusieurs
                D.label(X(ta) + 2, ry - rh - 7 - 8 * below_slot, lab,
                        size=6.4, color=col, bold=True, anchor="start",
                        bg=WHITE)
                below_slot += 1
        if mark is not None:
            mx = X(mark)
            D.d.add(Polygon(points=[mx, ry + 1, mx + 4, ry - rh / 2, mx,
                                    ry - rh - 1, mx - 4, ry - rh / 2],
                            fillColor=INK, strokeColor=INK))
            my_label = (ry + 6) if mark_pos == "above" else (ry - rh - 7)
            D.label(mx, my_label, mark_lab, size=6.2, color=INK, bold=True,
                    bg=WHITE)

    # axe
    ay = top - 4 * (rh + rgap) - 6
    D.d.add(Line(x0, ay, x1, ay, strokeColor=GREY, strokeWidth=0.8))
    for t in range(0, 22, 5):
        D.d.add(Line(X(t), ay, X(t), ay - 4, strokeColor=GREY, strokeWidth=0.8))
        D.label(X(t), ay - 12, f"{t} s", size=6.6, color=GREY)

    # SLO
    sx = X(5)
    D.d.add(Line(sx, H - 18, sx, ay, strokeColor=GREEN, strokeWidth=1.4,
                 strokeDashArray=[5, 3]))
    D.chip(sx, H - 12, "cible SLO : bascule < 5 s", GREEN_BG, GREEN,
           tcolor=GREEN, fs=7.2, bold=True)
    return D.add()


# ===========================================================================
# Schéma 8 — Mesh WireGuard intra-site
# ===========================================================================
def diag_mesh() -> Drawing:
    D = Dia(USABLE, 212)
    H = D.h
    # conteneur site
    sx, sy, sw, sh = 12, 34, 292, 152
    D.d.add(Rect(sx, sy, sw, sh, rx=8, ry=8, fillColor="#F7FAFC",
                 strokeColor=MID, strokeWidth=1.1))
    D.label(sx + sw / 2, sy + sh - 12, "SITE A — 10.10.1.0/24 (intra-site)",
            size=8, color=ACCENT, bold=True)

    nodes = [("node-1 · .1", 68, 118), ("node-2 · .2", 158, 128),
             ("node-3 · .3", 158, 62)]
    pos = []
    for name, nx, ny in nodes:
        D.box(nx - 44, ny - 14, 88, 28, [name], fill=BLUE_BG, stroke=BLUE,
              fs=7.4)
        pos.append((nx, ny))
    for i in range(3):
        for j in range(i + 1, 3):
            (x1, y1), (x2, y2) = pos[i], pos[j]
            dx, dy = x2 - x1, y2 - y1
            L = math.hypot(dx, dy)
            ux, uy = dx / L, dy / L
            D.arrow([(x1 + ux * 46, y1 + uy * 18), (x2 - ux * 46, y2 - uy * 18)],
                    color=BLUE, dash=[5, 3], sw=1.1)
    D.label(sx + sw / 2, 44, "tunnels WireGuard : etcd · Patroni · WAL circulent ici",
            size=6.8, color=BLUE)

    # registre etcd
    rx = 330
    D.box(rx, 92, 152, 52,
          ["Registre etcd", "/amane/mesh/nodes/<nom>",
           "clés PUBLIQUES seulement"],
          fill=LIGHT, stroke=ACCENT, fs=7.2)
    D.elbow(rx, 118, sx + sw, 118, 118, color=ACCENT, dash=[3, 2], double=True)
    D.label((rx + sx + sw) / 2, 124, "publish / discover", size=6.6,
            color=ACCENT, bg=WHITE)

    D.box(rx, 34, 152, 40,
          ["Clé privée : fichier local 0600",
           "jamais dans etcd, jamais loggée"],
          fill=RED_BG, stroke=RED, fs=7.2)

    # règles
    D.label(USABLE / 2, 14,
            "AllowedIPs /32 strict (0.0.0.0/0 refusé au rendu) · PersistentKeepalive 25 s (NAT) · "
            "IP virtuelle stable indépendante du rôle → aucun rechiffrement réseau au failover",
            size=7.0, color=INK)
    return D.add()


# ===========================================================================
# Schéma 9 — Trace distribuée W3C
# ===========================================================================
def diag_trace() -> Drawing:
    D = Dia(USABLE, 186)
    H = D.h
    cols = [("Client / SDK", 70), ("Orchestrator — nœud 1", 246),
            ("Orchestrator — nœud 2 (pair)", 422)]
    cw = 128
    box_y = H - 58
    # flèches et libellés au-dessus des boîtes
    D.arrow([(cols[0][1] + cw / 2, H - 14), (cols[1][1] - cw / 2, H - 14)],
            color=BLUE, sw=1.2)
    D.label((cols[0][1] + cols[1][1]) / 2, H - 7,
            "metadata gRPC : traceparent 00-<trace-id>-<span-1>-01",
            size=6.6, color=BLUE, bg=WHITE)
    D.arrow([(cols[1][1] + cw / 2, H - 14), (cols[2][1] - cw / 2, H - 14)],
            color=BLUE, sw=1.2)
    D.label((cols[1][1] + cols[2][1]) / 2, H - 24,
            "traceparent 00-<même trace-id>-<span-2>-01", size=6.6,
            color=BLUE, bg=WHITE)
    for name, cx in cols:
        D.box(cx - cw / 2, box_y, cw, 28, [name], fill=GREY_BG, stroke=GREY,
              fs=7.6)

    logs = [
        ('{"msg":"rpc","method":"/Write",', '"trace_id":"a1…","span_id":"s1"}'),
        ('{"msg":"rpc","method":"/PushDelta",',
         '"trace_id":"a1…","span_id":"s2"}'),
    ]
    for (name, cx), (l1, l2) in zip(cols[1:], logs):
        D.d.add(Rect(cx - cw / 2 - 6, 74, cw + 12, 26, rx=3, ry=3,
                     fillColor="#F4F6F8", strokeColor=MID, strokeWidth=0.7))
        D.txt(cx - cw / 2, 92, l1, size=6.0, font=F["mono"])
        D.txt(cx - cw / 2, 82, l2, size=6.0, font=F["mono"])
        D.d.add(Line(cx, box_y, cx, 100, strokeColor=MID, strokeWidth=0.8,
                     strokeDashArray=[2, 2]))
    D.d.add(Rect(cols[0][1] - cw / 2 - 6, 74, cw + 12, 26, rx=3, ry=3,
                 fillColor=AMBER_BG, strokeColor=AMBER, strokeWidth=0.7))
    D.txt(cols[0][1] - cw / 2 + 4, 92, "aucune trace entrante ?", size=6.6,
          font=F["mono"])
    D.txt(cols[0][1] - cw / 2 + 4, 82, "→ le serveur fait Root()", size=6.6,
          font=F["mono"])

    D.label(USABLE / 2, 46,
            "Le même trace-id relie les logs des deux nœuds : un Write et sa "
            "réplication deviennent traçables de bout en bout.",
            size=7.4, color=INK)
    D.label(USABLE / 2, 33,
            "Zéro dépendance OTel : format W3C standard — un SDK OTLP se "
            "branche plus tard sans changer les points d'ancrage.",
            size=7.2, color=GREY)
    D.label(USABLE / 2, 20,
            "Chaque span ajoute aussi duration_ms aux logs JSON (slog) : "
            "corrélation du timing failover / lease entre nœuds.",
            size=7.2, color=GREY)
    return D.add()


# ===========================================================================
# Schéma 10 — Pyramide de tests
# ===========================================================================
def diag_pyramid() -> Drawing:
    D = Dia(USABLE, 196)
    layers = [
        ("CHAOS & MESURES RÉELLES — failover crash/partition vs SLO < 5 s, fencing, zéro perte",
         RED_BG, RED, 300),
        ("E2E gRPC TCP RÉEL — propagation PushDelta de bout en bout, contrats B↔C / A↔C (module tests/ isolé)",
         AMBER_BG, AMBER, 372),
        ("INTÉGRATION etcd RÉEL — élection/fencing, superviseur vs faux Patroni, garde anti-partition",
         BLUE_BG, BLUE, 420),
        ("UNITAIRES — fake/mem, rapides, -count=1 -race (CRDT, detector, mesh, telemetry…)",
         GREEN_BG, GREEN, 452),
    ]
    y = 30
    for i, (txt_, fill, stroke, w) in enumerate(reversed(layers)):
        h = 30
        D.d.add(Rect((USABLE - w) / 2, y, w, h, rx=5, ry=5, fillColor=fill,
                     strokeColor=stroke, strokeWidth=1.0))
        words = txt_.split(" — ")
        D.label(USABLE / 2, y + h - 10, words[0], size=7.6, color=stroke,
                bold=True)
        D.label(USABLE / 2, y + 8, " — ".join(words[1:]), size=6.6, color=INK)
        y += h + 6
    D.label(USABLE / 2, 16,
            "Portes CI : build + vet · couverture agrégée ≥ 70 % (plancher 60 % par paquet) · "
            "buf lint/breaking · testcontainers · govulncheck + gosec · benchmarks vs baseline",
            size=7.0, color=GREY)
    return D.add()


# ===========================================================================
# Extraction du contrat proto (sources réelles)
# ===========================================================================
def parse_proto(path: Path) -> list[tuple[str, str, str, str]]:
    text = path.read_text(encoding="utf-8")
    out = []
    for m in re.finditer(r"//\s*(.*?)\n\s*rpc\s+(\w+)\((\w+)\)\s+returns\s+\((\w+)\);",
                         text, re.S):
        comment, rpc, req, resp = [g.strip() for g in m.groups()]
        first = comment.split("\n")[-1]
        first = re.sub(r"\s*\(.*?\)\s*$", "", first)
        out.append((rpc, req, resp, first.rstrip(".")))
    return out


RPCS = parse_proto(PROTO_FILE)


# ===========================================================================
# En-tête / pied de page
# ===========================================================================
def _footer(canv, doc):
    canv.saveState()
    canv.setStrokeColor(MID)
    canv.setLineWidth(0.6)
    canv.line(MARGIN, 13 * mm, PAGE_W - MARGIN, 13 * mm)
    canv.setFont(F["base"], 7.5)
    canv.setFillColor(GREY)
    canv.drawString(MARGIN, 9.5 * mm,
                    "AMANE — Mission C · La logique globale (orchestrator-go)")
    canv.drawRightString(PAGE_W - MARGIN, 9.5 * mm, f"page {doc.page}")
    canv.restoreState()


def _cover_bg(canv, doc):
    canv.saveState()
    canv.setFillColor(ACCENT)
    canv.rect(0, PAGE_H - 26 * mm, PAGE_W, 26 * mm, stroke=0, fill=1)
    canv.setFillColor(WHITE)
    canv.setFont(F["bold"], 11)
    canv.drawString(MARGIN, PAGE_H - 16 * mm, "AMANE")
    canv.setFont(F["base"], 9)
    canv.drawString(MARGIN, PAGE_H - 21 * mm,
                    "Plateforme souveraine — infrastructure & résilience")
    _footer(canv, doc)
    canv.restoreState()


# ===========================================================================
# Assemblage du document
# ===========================================================================
story = []
P = lambda t, s=st_body: story.append(Paragraph(t, s))

# ---------------------------------------------------------------- couverture
story.append(Spacer(1, 34 * mm))
P("La logique globale<br/>de la Mission C", st_title)
story.append(Spacer(1, 5 * mm))
P("Comprendre, d'un seul tenant, ce que construit <b>orchestrator-go</b> : "
  "un orchestrateur distribué qui garantit un cluster résilient — un seul "
  "écrivain, une réplication multi-site qui converge, une base de données "
  "haute disponibilité qui rebascule en moins de 5 secondes.", st_subtitle)
story.append(Spacer(1, 12 * mm))

pillars = [
    ["Un seul écrivain", "consensus etcd + fencing par lease — anti split-brain"],
    ["Convergence AP", "delta-CRDT max-merge : doublons et coupures inoffensifs"],
    ["HA < 5 s", "superviseur maison : détecter vite, laisser Patroni décider"],
    ["Preuve continue", "tests à 3 niveaux + chaos + CI à seuils bloquants"],
]
cells = [[Paragraph(f"<b><font color='#FFFFFF'>{a}</font></b><br/>"
                    f"<font color='#DCE6F0' size='7.6'>{b}</font>",
                    _ps("pv", fontSize=9.5, leading=12.5))]
         for a, b in pillars]
tp = Table([[c[0] for c in cells]], colWidths=[USABLE / 4] * 4)
tp.setStyle(TableStyle([
    ("BACKGROUND", (0, 0), (0, 0), ACCENT),
    ("BACKGROUND", (1, 0), (1, 0), BLUE),
    ("BACKGROUND", (2, 0), (2, 0), colors.HexColor("#3E7CB1")),
    ("BACKGROUND", (3, 0), (3, 0), GREEN),
    ("VALIGN", (0, 0), (-1, -1), "TOP"),
    ("TOPPADDING", (0, 0), (-1, -1), 8),
    ("BOTTOMPADDING", (0, 0), (-1, -1), 9),
    ("LEFTPADDING", (0, 0), (-1, -1), 8),
    ("RIGHTPADDING", (0, 0), (-1, -1), 8),
]))
story.append(tp)
story.append(Spacer(1, 14 * mm))
P(f"Document généré le {date.today().strftime('%d/%m/%Y')} depuis le code du dépôt "
  f"(sources : orchestrator-go/, proto/, scripts/, docker-compose.yml). "
  f"Tous les schémas sont vectoriels et régénérables :", st_small)
P(mono(".venv/bin/python docs/generate_mission_c_logique_pdf.py", 8), st_small)
story.append(PageBreak())

# ---------------------------------------------------------------- sommaire
P("Sommaire", st_h1)
toc = [
    ("1", "Le problème que résout Mission C", "les quatre exigences et leur traduction technique"),
    ("2", "Vue d'ensemble : ce que fait le système", "architecture physique, composants, plan mémoire etcd"),
    ("3", "Le contrat gRPC : la frontière versionnée", "proto partagé B↔C / A↔C, règle de compatibilité buf"),
    ("4", "Un seul écrivain : consensus et fencing", "élection par lease, perte de lease = verrouillage"),
    ("5", "Le chemin d'écriture : gated et idempotent", "la garde IsLeader() avant tout accès au journal"),
    ("6", "Réplication multi-site : le delta-CRDT", "totaux cumulés, fusion max, propagateur périodique"),
    ("7", "Haute disponibilité < 5 s : le superviseur", "détection rapide, garde anti-partition, mesures"),
    ("8", "Mesh WireGuard intra-site", "registre public, config /32, secrets locaux"),
    ("9", "Sécurité transverse", "TLS 1.3 + mTLS, zéro secret en clair, payload chiffré"),
    ("10", "Observabilité : la trace distribuée W3C", "traceparent de nœud en nœud, logs corrélés"),
    ("11", "La preuve par les tests et la CI", "pyramide de tests, chaos, seuils bloquants"),
    ("12", "Synthèse : six principes directeurs", "la logique d'ensemble en une page"),
]
rows = [["#", "Section", "Ce que vous y apprendrez"]]
for n, t, d in toc:
    rows.append([n, f"<b>{esc(t)}</b>", d])
story.append(table(rows, [10 * mm, 78 * mm, 89 * mm]))
story.append(Spacer(1, 6 * mm))
P("<b>Comment lire ce document.</b> Chaque section suit le même schéma : "
  "<i>quel problème</i>, <i>quelle réponse</i>, <i>comment ça marche</i> "
  "(schéma), <i>où dans le code</i>. Les chiffres de bascule proviennent des "
  "mesures exécutables (<font face='Courier' size='8'>scripts/*_measure.sh</font>) "
  "sur la stack de développement réelle (docker-compose : etcd + Patroni/Spilo).",
  st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 1
P("1. Le problème que résout Mission C", st_h1)
P("La mission porte l'<b>infrastructure et la résilience</b> du cluster AMANE. "
  "Les autres missions construisent des fonctionnalités ; Mission C garantit "
  "qu'elles tiennent debout quand tout va mal : crash d'un nœud, coupure "
  "réseau entre sites, machine compromise, latence.", st_lead)
P("Quatre exigences structurent tout le code — et chacune a reçu une réponse "
  "mécanique précise plutôt qu'une promesse :", st_body)
rows = [["Exigence", "Réponse apportée", "Section"],
        ["Il ne doit jamais y avoir deux écrivains simultanés "
         "(split-brain = corruption silencieuse)",
         "Élection du leader par lease etcd + <b>fencing applicatif</b> : "
         "une écriture sur un nœud non-leader est refusée avant tout accès au journal", "4–5"],
        ["Les stocks doivent converger entre sites malgré un réseau lent ou coupé",
         "<b>delta-CRDT</b> (totaux cumulés, fusion max) : ordre, doublons et "
         "trous deviennent mathématiquement inoffensifs", "6"],
        ["Une panne du primaire PostgreSQL doit être absorbée en moins de 5 s, sans perte",
         "<b>Superviseur maison</b> : détection en 0,8 s, déverrouillage du lock "
         "Patroni, POST /failover — Patroni reste l'autorité", "7"],
        ["Rien ne doit être cassable en silence ni illisible en production",
         "Contrat proto verrouillé par CI, tests à trois niveaux, chaos "
         "planifié, trace distribuée W3C, mTLS partout", "3, 9–11"]]
story.append(table(rows, [58 * mm, 104 * mm, 15 * mm]))
story.append(Spacer(1, 3 * mm))
P("Le fil rouge : <b>chaque garantie est prouvée par un test ou une mesure "
  "ré-exécutable</b>, jamais affirmée. C'est pourquoi la dernière section de "
  "ce document parle autant de tests que les premières parlent d'algorithme.",
  st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 2
P("2. Vue d'ensemble : ce que fait le système", st_h1)
P("<b>L'unité de déploiement est le site</b> (un atelier, une boutique en "
  "ligne…). Chaque site embarque la pile complète : l'orchestrateur Go expose "
  "l'API gRPC, etcd tient le consensus local, PostgreSQL (piloté par Patroni) "
  "porte les données avec un réplica synchrone, et un mesh WireGuard chiffre "
  "tout le trafic interne. Les sites se répliquent entre eux par un flux "
  "asynchrone de deltas — tolérant aux coupures.", st_lead)
story.append(diag_architecture())
P("Figure 1 — Architecture globale : deux sites autonomes reliés par la "
  "réplication CRDT. Les flèches grises montrent qui contrôle quoi.",
  st_caption)
story.append(PageBreak())

P("Les briques logicielles", st_h2)
rows = [["Paquet Go", "Rôle dans une phrase", "Interface"]
        ] + [
    ["consensus/", "Élection du leader (lease 30 s), registre des machines, calcul du quorum", "etcd v3"],
    ["grpcserver/", "Expose les 6 RPC du contrat ; applique le fencing et route vers le relay/journal", "gRPC"],
    ["replication/", "PN-Counter CRDT (relay), journal append-only idempotent, propagateur périodique", "interne"],
    ["supervisor/", "Détecte un crash de primary en < 1 s et déclenche le failover Patroni", "REST + etcd"],
    ["mesh/", "Découverte des pairs via etcd et génération de wg0.conf (0600)", "WireGuard"],
    ["telemetry/", "Propage l'en-tête W3C traceparent et corrélationne les logs JSON", "gRPC metadata"],
    ["cmd/orchestrator", "Câblage : TLS, interceptors, leadership, superviseur, propagateurs", "main()"],
    ["cmd/wgmesh", "Outil runtime du mesh : publication, découverte, écriture de wg0.conf", "CLI"],
]
story.append(table(rows, [34 * mm, 112 * mm, 31 * mm]))

P("Le plan mémoire du cluster : six familles de clés etcd", st_h2)
P("Tout l'état partagé vit sous des préfixes etcd explicites — c'est la "
  "mémoire commune du cluster. Retenir ces clés, c'est comprendre le système :", st_body)
rows = [["Clé etcd", "Qui l'écrit", "Signification"],
        ["/amane/leader", "consensus.Leadership", "qui détient le droit d'écrire (fencing applicatif)"],
        ["/amane/members/<machine>", "Enroll / NotifyRevocation", "registre des machines enrôlées → quorum"],
        ["/service/<scope>/leader", "Patroni (DCS)", "lock du primary PostgreSQL — autorité de promotion"],
        ["/amane/supervisor/hb/<nœud>", "agent heartbeat local", "« mon nœud est vivant et primary » (lease 2 s)"],
        ["/amane/supervisor/force-right", "superviseurs", "lock « droit de forcer » : un seul acteur agit"],
        ["/amane/mesh/nodes/<nom>", "outil wgmesh", "info publique WireGuard (jamais de clé privée)"]]
story.append(table(rows, [52 * mm, 40 * mm, 85 * mm]))
story.append(Spacer(1, 2 * mm))
P("Distinction cruciale : <b>/amane/leader</b> (fencing applicatif, géré par "
  "notre code) et <b>/service/&lt;scope&gt;/leader</b> (lock PostgreSQL, géré "
  "par Patroni) sont deux verrous différents. Le superviseur libère le second "
  "au moment du failover — jamais l'inverse (section 7).", st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 3
P("3. Le contrat gRPC : la frontière versionnée", st_h1)
P("Mission C est un serveur ; les Missions A et B sont ses clients. Tout "
  "passe donc par un contrat protobuf partagé "
  "(<font face='Courier' size='8'>proto/amane/framework/v1/framework.proto</font>), "
  "généré avec buf et committé. La règle est absolue : <b>ne jamais "
  "renuméroter un champ ni changer son type</b> — cela casserait en silence "
  "les SDK déjà déployés. La CI rejette toute rupture (buf breaking).", st_lead)
P("Les six opérations du service", st_h2)
rows = [["RPC", "Rôle", "Sens"]]
for rpc, req, resp, com in RPCS:
    sens = "A↔C" if rpc in ("Enroll", "NotifyRevocation") else (
        "B↔C" if rpc in ("Write", "Read") else "interne/transverse")
    rows.append([mono(rpc, 7.8), esc(com), sens])
story.append(table(rows, [34 * mm, 106 * mm, 27 * mm]))
story.append(Spacer(1, 3 * mm))
P("Deux conventions traversent ce contrat : les erreurs sont toujours des "
  "<font face='Courier' size='8'>status.Error(codes.X)</font> gRPC propres "
  "(jamais d'erreur brute), et le champ <font face='Courier' size='8'>encrypted_payload</font> "
  "n'est jamais autre chose que du chiffré produit par la Mission A — le "
  "serveur C ne voit jamais de clair et ne logue que des longueurs.", st_body)
story.append(Spacer(1, 2 * mm))
P("Ajouter un RPC est sûr quand il est <b>additif</b> : c'est comme cela "
  "qu'est né PushDelta (section 6) — nouveaux messages, nouveaux numéros de "
  "champs, anciens intacts, code généré committé.", st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 4
P("4. Un seul écrivain : consensus et fencing", st_h1)
P("<b>Le problème.</b> Si deux nœuds se croient simultanément maîtres des "
  "écritures (après une micro-coupure réseau, par exemple), chacun numérotera "
  "les entrées à sa façon : corruption silencieuse du journal. Il faut un "
  "mécanisme où perdre le droit d'écrire est <i>instantané et irréfutable</i>.", st_lead)
P("<b>La réponse : une lease etcd.</b> Le droit d'écrire est matérialisé par "
  "une session dont la vie dépend d'un keepalive continu. Tant que le nœud "
  "bat le cœur, il est leader ; dès que la lease expire — crash, coupure, "
  "arrêt — la clé disparaît d'elle-même et le nœud sait localement "
  "(<font face='Courier' size='8'>IsLeader()</font> passe à false) qu'il doit "
  "refuser d'écrire. Aucune négociation, aucune fenêtre ambiguë.", st_body)
story.append(diag_fencing())
P("Figure 2 — Cycle de vie du leadership : campagne → leader → fencing → "
  "re-campagne. La boucle tourne tant que le processus vit.", st_caption)
story.append(Spacer(1, 3 * mm))
P("<b>Pourquoi c'est sûr.</b> L'élection repose sur l'Election d'etcd "
  "(concurrency.Election) : exactement un gagnant par instance de lease. Un "
  "ancien leader qui revient après une partition ne peut pas récupérer le "
  "trône tant qu'une autre session le détient ; et pendant qu'il était "
  "isolé, son IsLeader() était déjà false — il n'a rien écrit. C'est le "
  "fencing « lease-based » classique, appliqué au chemin d'écriture gRPC.", st_body)
bullets_list = bullets([
    "<b>Côté code :</b> consensus/lease.go (Run/IsLeader), clé /amane/leader, TTL 30 s.",
    "<b>Côté serveur :</b> grpcserver/server.go — Write consulte WithLeadership avant le journal.",
    "<b>Côté preuve :</b> TestLeadershipFencingAgainstEtcd (etcd réel) : l'ancien leader ne peut plus écrire après réélection.",
])
story.extend(bullets_list)
story.append(PageBreak())

# ---------------------------------------------------------------- section 5
P("5. Le chemin d'écriture : gated et idempotent", st_h1)
P("Le RPC Write est le point unique où les données entrent. Deux protections "
  "s'y cumulent : le <b>fencing</b> (section 4) qui filtre les non-leaders, "
  "et l'<b>idempotence</b> du journal qui rend les rejeux inoffensifs.", st_lead)
story.append(diag_write_path())
P("Figure 3 — Chemin d'écriture : la garde IsLeader() passe avant tout accès "
  "au journal ; un rejeu (même journal/site/op_seq) rend le seq déjà committé.",
  st_caption)
story.append(Spacer(1, 3 * mm))
P("<b>Le journal en deux idées.</b> Un map <font face='Courier' size='8'>journal_id → entrées</font> "
  "append-only, et une clé de déduplication <font face='Courier' size='8'>(journal_id, site_id, op_seq)</font> : "
  "si le client renvoie la même opération (timeout, retry réseau), le serveur "
  "répond le seq déjà committé au lieu de dupliquer. Les seq attribués sont "
  "monotones par journal — le contrat promis aux lecteurs.", st_body)
P("<b>Et la lecture ?</b> Read n'est pas gated : lire des données un peu "
  "anciennes sur un réplica vaut mieux que ne plus pouvoir lire du tout "
  "(cohérence ultime assumée pour les lectures). Seule l'écriture modifie "
  "l'histoire, donc seule l'écriture exige l'unicité du leader.", st_body)

P("Ce qui circule — et ce qui ne circule jamais", st_h2)
rows = [["Donnée", "Traitement côté C"],
        ["payload chiffré", "stocké tel quel, logué en longueur uniquement (jamais en clair)"],
        ["op_seq client", "servit à la déduplication ; le seq committé est attribué par le serveur"],
        ["clé publique AK (Enroll)", "empreinte SHA-256 (8 octets) stockée ; la clé elle-même n'est pas persistée"],
        ["identifiant de clé révoquée", "seul le fingerprint circule dans NotifyRevocation"]]
story.append(table(rows, [52 * mm, 125 * mm]))
story.append(PageBreak())

# ---------------------------------------------------------------- section 6
P("6. Réplication multi-site : le delta-CRDT (mode AP)", st_h1)
P("<b>Le problème.</b> Les quantités de stock doivent être identiques sur "
  "tous les sites, mais les liens inter-sites sont lents et parfois coupés. "
  "Exiger une écriture synchrone entre sites serait une sentence : chaque "
  "coupure figerait tout. On choisit donc la disponibilité (mode AP) — à "
  "condition que la convergence soit <i>mathématiquement</i> garantie.", st_lead)
P("<b>L'idée géniale mais simple des CRDT.</b> Un delta n'emporte pas « +2 "
  "pièces » (un incrément relatif, qui casse si répété ou réordonné) mais le "
  "<b>total cumulé</b> de l'émetteur depuis son origine : « mes entrées "
  "totales = 7 ». Fusionner, c'est prendre le max par nœud émetteur. Max est "
  "commutatif, associatif et idempotent : peu importe l'ordre d'arrivée, les "
  "doublons ou les trous — tous les nœuds finissent avec exactement le même "
  "état. Ce n'est pas une espérance, c'est un théorème.", st_body)
story.append(diag_crdt())
P("Figure 4 — Exemple de convergence : +5 puis +7 d'un côté, −3 de l'autre ; "
  "chaque nœud fusionne par max et obtient 4, quel que soit le trafic réseau.",
  st_caption)
story.append(PageBreak())
P("<b>La livraison : un propagateur, pas une prière.</b> Chaque nœud garde "
  "ses deltas non accusés dans un <i>pending</i> et un Propagator les pousse "
  "toutes les secondes vers chaque pair configuré. L'accusé porte la dernière "
  "seq appliquée par le pair : il purge le pending (Confirm). Une erreur "
  "réseau n'efface rien — le tick suivant retente. Au pire, on retarde ; "
  "jamais on ne perd.", st_body)
story.append(diag_propagator())
P("Figure 5 — Boucle de livraison d'un delta, de Add() jusqu'au gc du pending.",
  st_caption)
story.append(Spacer(1, 3 * mm))
rows = [["Situation réseau", "Effet sur le système"],
        ["delta reçu deux fois", "aucun : max({7},{7}) = 7 (idempotence)"],
        ["deltas reçus en désordre", "aucun : le max ne dépend pas de l'ordre (commutativité)"],
        ["seq manquants (trous)", "aucun : la seq ne sert qu'à l'accusé côté émetteur"],
        ["pair injoignable des heures", "retard seulement : le pending grossit puis se vide à la reconnexion"],
        ["nos propres deltas revenus (réflexion mesh)", "ignorés explicitement dans Accept()"]]
story.append(table(rows, [70 * mm, 107 * mm]))
story.append(Spacer(1, 3 * mm))
P("<b>Frontières honnêtes.</b> Ce compteur PN-CRDT sert aux quantités de "
  "stock — un domaine où la convergence compte plus que l'instantanéité. Le "
  "journal d'écriture, lui, reste centralisé par leader (section 5) : deux "
  "régimes de cohérence choisis sciemment selon la donnée. Preuve live du "
  "repo : e-shop +5 puis +7 → ack seq 2, valeur 7 ; atelier −3 → valeur 4 "
  "partout.", st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 7
P("7. Haute disponibilité < 5 s : le superviseur (option C)", st_h1)
P("<b>Le problème.</b> PostgreSQL bascule grâce à Patroni, mais Patroni "
  "s'appuie sur une lease DCS dont le plancher est ttl ≥ 20 s (validé dans "
  "patroni/config.py, quel que soit le DCS). En cas de crash brutal, la "
  "promotion naturelle attend donc ~21 s — quatre fois trop pour le SLO. "
  "Abaisser ce TTL affaiblirait la protection anti-split-brain : rejeté.", st_lead)
P("<b>La réponse retenue : détecter vite sans décider à la place de "
  "l'expert.</b> Un petit superviseur en Go, déployé sur chaque nœud, sonde "
  "toutes les 500 ms l'API REST Patroni et publie un <b>heartbeat propre</b> "
  "dans etcd avec une lease de 2 s — indépendante du plancher de 20 s. Quand "
  "crash il y a (REST muet ET heartbeat disparu), il déverrouille la situation "
  "en deux gestes : supprimer le lock Patroni du défunt, puis demander la "
  "promotion. <b>Patroni reste seul juge de qui promouvoir</b> (réplica "
  "synchrone → zéro perte) : le superviseur n'est pas une couche HA "
  "parallèle, juste un déclencheur rapide.", st_body)
story.append(PageBreak())
story.append(diag_supervisor_flow())
P("Figure 6 — L'arbre de décision à chaque tick. La garde anti-partition "
  "(ambre) distingue un nœud mort d'un nœud vivant mais isolé.", st_caption)
story.append(PageBreak())

P("La garde anti-partition : le cœur subtil du dispositif", st_h2)
P("Couper le REST d'un primary ne le tue pas. Si le superviseur forçait sur "
  "la seule foi du REST muet, un simple incident réseau créerait deux "
  "primaires. D'où la règle : <b>forcer exige DEUX preuves de mort</b> — REST "
  "muet ET heartbeat etcd absent/périmé. Un primary isolé continue de battre "
  "son cœur : la garde bloque, Patroni tranche.", st_body)
P("Subtilité de déploiement : chaque superviseur ne publie/ne supprime le "
  "heartbeat que de <b>son propre nœud</b> (LocalNode). Sinon, quand SON lien "
  "REST tombe, il effacerait le heartbeat du primary et neutraliserait la "
  "garde. Ce piège a été attrapé par un test d'intégration avant d'atteindre "
  "la production.", st_body)

P("Pourquoi supprimer le lock change tout", st_h2)
P("Un POST /failover « force » seul suffit rarement : tant que le lock "
  "Patroni existe, la promotion reste bornée par l'expiration de la lease "
  "(≥ 20 s). Supprimer explicitement la clé /service/&lt;scope&gt;/leader — "
  "safe car le heartbeat stale prouve que le détenteur ne se battra pas — "
  "libère la promotion immédiatement. Le candidat choisi est un réplica "
  "joignable, prioritairement sync_state=sync (zéro perte).", st_body)

P("Les mesures qui closent le débat", st_h2)
story.append(diag_timeline())
P("Figure 7 — Temps de bascule mesurés sur la stack réelle (scripts/"
  "failover_measure.sh, switchover_measure.sh). Fencing et zéro perte "
  "vérifiés à chaque exécution.", st_caption)
story.append(Spacer(1, 3 * mm))
rows = [["Scénario", "Détection", "Failover", "Writable", "Cible < 5 s"],
        ["Switchover planifié", "—", "~2,2 s", "~2,4 s", "tenue"],
        ["Crash SIGKILL + superviseur", "0,8 s", "3,2 s", "4,0 s", "tenue"],
        ["Partition totale + superviseur", "0,7 s", "3,0 s", "3,4 s", "tenue"],
        ["Crash sans superviseur", "—", "~21 s", "~22 s", "hors SLO (plancher Patroni)"]]
story.append(table(rows, [62 * mm, 24 * mm, 26 * mm, 26 * mm, 39 * mm]))
story.append(Spacer(1, 2 * mm))
P("Invariant vérifié à chaque mesure : synchronous_commit=on, standby "
  "synchrone, l'ancien primary ne recapture jamais (fencing), zéro "
  "transaction acquittée perdue.", st_small)
story.append(PageBreak())

# ---------------------------------------------------------------- section 8
P("8. Mesh WireGuard intra-site", st_h1)
P("Le trafic sensible d'un site — réplication WAL, heartbeats etcd, API "
  "Patroni — circule sur un VPN WireGuard interne. L'inter-site, lui, reste "
  "sur du TLS sortant : le mesh n'est volontairement <b>pas</b> la colonne "
  "vertébrale mondiale, juste le bouclier local.", st_lead)
story.append(diag_mesh())
P("Figure 8 — Le runtime wgmesh publie l'information publique dans etcd, "
  "découvre les pairs et régénère wg0.conf uniquement si le contenu change.", st_caption)
story.append(Spacer(1, 3 * mm))
P("Trois choix de sécurité dignes d'attention. D'abord, <b>AllowedIPs /32</b> : "
  "chaque pair n'autorise que l'IP virtuelle précise de l'autre — une route "
  "globale 0.0.0.0/0 serait rejetée au rendu (ErrDangerousAllowedIPs), "
  "impossible de détourner le trafic par une config bâclée. Ensuite, l'<b>IP "
  "virtuelle stable par nœud</b> (10.10.&lt;site&gt;.&lt;index&gt;) : un "
  "failover change un rôle, jamais une adresse — rien à reconfigurer au "
  "moment critique. Enfin, la <b>clé privée reste locale</b> : fichier 0600, "
  "jamais dans etcd, jamais dans les logs ; seul le matériel public circule.", st_body)

# ---------------------------------------------------------------- section 9
P("9. Sécurité transverse : TLS 1.3, mTLS, secrets", st_h1)
P("La posture tient en une phrase : <b>chiffrer par défaut, ne jamais exposer "
  "un secret, minimiser ce qui est cru.</b>", st_lead)
rows = [["Surface", "Règle appliquée", "Preuve"],
        ["Transport gRPC", "TLS 1.3 minimum (MinVersion=1.3) — TLS 1.2 rejeté", "TestCredentials ; openssl s_client -tls1_2 échoue"],
        ["Authentification", "mTLS : certificat client exigé et vérifié contre la CA interne", "ClientAuth=RequireAndVerifyClientCert"],
        ["Payloads", "chiffrés par la Mission A ; C ne voit/logue que du chiffré", "revues de code + tests"],
        ["Clés AK (enrôlement)", "seules clés publiques circulent ; empreinte SHA-256 comme ID", "consensus.Fingerprint"],
        ["PKI tls/", "réservée au développement (clés 0600, jamais committée en prod)", "règle AGENTS.md n° 6"],
        ["Production (à venir)", "AC racine hors ligne, ACs intermédiaires par site, OCSP/CRL, DNSSEC sortant", "planifié hors POC"]]
story.append(table(rows, [34 * mm, 88 * mm, 55 * mm]))
story.append(Spacer(1, 2 * mm))
P("Le principe directeur : une règle de sécurité qui n'est pas vérifiée par "
  "un test n'existe pas. Chaque ligne ci-dessus renvoie vers un test ou une "
  "commande de contre-vérification exécutable.", st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 10
P("10. Observabilité : la trace distribuée W3C", st_h1)
P("Dans un système à trois sites, « les logs » d'un incident sont éparpillés "
  "sur plusieurs machines. La réponse standard est OpenTelemetry ; la réponse "
  "Mission C est une version <b>forward-path</b> sans dépendance : le format "
  "W3C traceparent, transporté dans les métadonnées gRPC, avec génération "
  "d'une racine si l'appelant n'en porte pas. Le jour où un vrai SDK OTLP "
  "est branché, aucun point d'ancrage ne change.", st_lead)
story.append(diag_trace())
P("Figure 9 — Un trace-id unique traverse les nœuds ; chaque saut crée un "
  "span enfant et enrichit les logs JSON (slog) de durée et d'identifiants.", st_caption)
story.append(Spacer(1, 3 * mm))
P("Trois intercepteurs habillent chaque appel serveur, dans l'ordre : "
  "<b>télémétrie</b> (extrait/injecte le traceparent), <b>logging</b> "
  "(méthode, code, latence, trace_id/span_id), <b>recovery</b> (transforme un "
  "panic en codes.Internal — le serveur ne meurt jamais pour un handler "
  "fautif). Côté client, l'intercepteur réinjecte l'en-tête vers le pair : "
  "c'est lui qui rend le chemin PushDelta traçable de bout en bout.", st_body)

# ---------------------------------------------------------------- section 11
P("11. La preuve par les tests et la CI", st_h1)
P("La philosophie : <b>chaque garantie précédente a un test qui la trahit "
  "si on la casse.</b> Les tests sont organisés en pyramide — vite et souvent "
  "en bas, réellement et rarement en haut — et la CI refuse de fusionner si "
  "un étage faiblit.", st_lead)
story.append(diag_pyramid())
P("Figure 10 — Pyramide de tests et portes CI associées.", st_caption)
story.append(Spacer(1, 3 * mm))
P("Les tests d'intégration utilisent un <b>etcd réel</b> (variable "
  "AMANE_TEST_ETCD ; testcontainers en CI) — mocker etcd testerait nos "
  "hypothèses sur etcd, pas etcd. Les tests cross-mission vivent dans un "
  "<b>module Go séparé</b> (tests/) qui consomme le produit : ils verrouillent "
  "les contrats B↔C et A↔C tels qu'un client les voit, pas tels que le code "
  "interne les arrange.", st_body)
rows = [["Workflow CI", "Déclencheur", "Ce qu'il bloque"],
        ["proto-contract", "push/PR", "buf lint + breaking : toute rupture du contrat"],
        ["build-go", "push/PR", "build + vet + tests + couverture ≥ 70 % (plancher 60 %/paquet)"],
        ["integration-tests", "push/PR", "contrats B↔C / A↔C / write-gated sur etcd réel (testcontainers)"],
        ["chaos-tests", "PR consensus·replication·supervisor", "failover crash, garde anti-partition, fencing, convergence CRDT"],
        ["benchmark", "PR chauds + cron", "micro-bench vs baseline.json ; failover macro vs SLO < 5 s"],
        ["security-scan", "push/PR + cron", "govulncheck + gosec (high bloquant)"]]
story.append(table(rows, [36 * mm, 52 * mm, 89 * mm]))
story.append(Spacer(1, 2 * mm))
P("Le plancher de couverture <b>par paquet</b> (60 %) complète le seuil "
  "agrégé (70 %) : un paquet à 40 % ne peut plus être masqué par son voisin "
  "à 90 %. Mesures actuelles : replication 95,7 %, telemetry 93,2 %, "
  "grpcserver 85,3 %, mesh 83,2 %, supervisor 80,2 %, consensus 73,8 % — "
  "agrégat ~72 %. Objectif annoncé : 75 % puis 80 %.", st_body)
story.append(PageBreak())

# ---------------------------------------------------------------- section 12
P("12. Synthèse : six principes directeurs", st_h1)
P("Si l'on ne devait retenir que la logique — celle qui explique chaque "
  "ligne de code — ce seraient ces six principes :", st_lead)
principles = [
    ("1 · L'état critique a un propriétaire unique",
     "Le droit d'écrire est une lease, pas une opinion. Qui perd sa lease perd son droit, instantanément et sans ambiguïté (sections 4–5)."),
    ("2 · La convergence vaut mieux que la coordination",
     "Entre sites, on ne verrouille pas : on fusionne. Des totaux cumulés + un max par émetteur rendent les incidents réseau arithmétiquement inoffensifs (section 6)."),
    ("3 · Détecter vite, décider sobrement",
     "Le superviseur apporte la vitesse (0,8 s) ; Patroni conserve l'autorité (promotion synchrone, zéro perte). Deux preuves de mort avant tout forçage (section 7)."),
    ("4 · Les contrats sont sacrés",
     "Le proto est la loi commune : additif seulement, breaking interdit, vérifié en CI. Les erreurs sont typées gRPC, jamais ambigües (section 3)."),
    ("5 · Pas de secret exposé, jamais",
     "Clés privées locales 0600, payloads chiffrés de bout en bout, fingerprints au lieu de clés, mTLS obligatoire (sections 8–9)."),
    ("6 · Une affirmation sans test n'est pas une fonctionnalité",
     "Unitaires rapides, intégrations sur etcd réel, chaos planifié, benchmarks contre baseline, seuils bloquants — la confiance se construit, elle ne se déclare pas (section 11)."),
]
rows = []
for t, d in principles:
    rows.append([Paragraph(f"<b><font color='#1F3A5F'>{esc(t)}</font></b>", st_body),
                 Paragraph(d, st_body)])
tt = Table(rows, colWidths=[52 * mm, 125 * mm])
tt.setStyle(TableStyle([
    ("GRID", (0, 0), (-1, -1), 0.4, MID),
    ("VALIGN", (0, 0), (-1, -1), "TOP"),
    ("TOPPADDING", (0, 0), (-1, -1), 6),
    ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
    ("LEFTPADDING", (0, 0), (-1, -1), 6),
    ("RIGHTPADDING", (0, 0), (-1, -1), 6),
    ("ROWBACKGROUNDS", (0, 0), (-1, -1), [WHITE, LIGHT]),
]))
story.append(tt)
story.append(Spacer(1, 6 * mm))
P("Et la promesse finale du système, celle que les mesures valident à chaque "
  "exécution : <b>un cluster qui écrit à un seul endroit, se réplique partout, "
  "survit à la mort de son primaire en moins de cinq secondes sans perdre une "
  "transaction, et raconte tout ce qu'il fait dans des traces corrélées.</b>", st_lead)

# ---------------------------------------------------------------------------
doc = BaseDocTemplate(str(OUT), pagesize=A4,
                      leftMargin=MARGIN, rightMargin=MARGIN,
                      topMargin=16 * mm, bottomMargin=18 * mm,
                      title="AMANE Mission C — La logique globale",
                      author="Mission C (orchestrator-go)")
frame_cover = Frame(MARGIN, 18 * mm, USABLE, PAGE_H - 40 * mm, id="cover")
frame_norm = Frame(MARGIN, 18 * mm, USABLE, PAGE_H - 34 * mm, id="norm")
doc.addPageTemplates([
    PageTemplate(id="Cover", frames=[frame_cover], onPage=_cover_bg),
    PageTemplate(id="Normal", frames=[frame_norm], onPage=_footer),
])
story.insert(0, NextPageTemplate("Normal"))

doc.build(story)
print(f"OK → {OUT}")
