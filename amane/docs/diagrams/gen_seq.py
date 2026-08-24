#!/usr/bin/env python3
"""Génère les diagrammes de séquence Mission C en dot (Graphviz), technique
lifelines + rangées rank=same : chaque message devient une arête horizontale
entre deux lifelines (contraintes invisibles), le temps coule de haut en bas.

Usage : .venv/bin/python docs/diagrams/gen_seq.py   (produit seq_*.png)
"""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def esc(s: str) -> str:
    return s.replace('"', '\\"')


def sequence(
    name: str,
    title: str,
    parts: list[tuple[str, str, str]],  # (id, libellé, couleur)
    messages: list[tuple[str, str, str, str]],  # (émetteur, récepteur, libellé, style)
    notes: dict[int, str] | None = None,  # rangée (1-based) -> texte du pavé
) -> str:
    notes = notes or {}
    nrows = len(messages)
    cols = []
    for pid, _label, _color in parts:
        cols.append(pid)

    L: list[str] = []
    L.append(f"digraph {name} {{")
    L.append("  rankdir=TB;")
    L.append(f'  graph [fontname="Helvetica", fontsize=11, fontcolor="#0b3d66", '
             f'label="{esc(title)}", labelloc=t, pad=0.4, splines=polyline];')
    L.append('  node [fontname="Helvetica", fontsize=9, shape=point, width=0.02, height=0.02];')
    L.append('  edge [fontname="Helvetica", fontsize=8, color="#4a7196", arrowsize=0.7];')

    for pid, label, color in parts:
        L.append(f'  H_{pid} [label="{esc(label)}", shape=box, style="rounded,filled", '
                 f'fillcolor="{color}", color="#2a4a66", fontname="Helvetica-Bold", fontsize=10];')

    for pid, _label, _color in parts:
        prev = f"H_{pid}"
        for t in range(1, nrows + 1):
            cur = f"{pid}_{t}"
            L.append(f"  {cur} [label=\"\", shape=point, width=0.01, height=0.01];")
            L.append(f"  {prev} -> {cur} [style=invis, constraint=true];")
            prev = cur

    for t, (src, dst, label, style) in enumerate(messages, start=1):
        # le `#` est un commentaire dot : on quote les couleurs hex.
        style = re.sub(r"color=#([0-9a-fA-F]{3,6})", r'color="#\1"', style)
        L.append(f"  {src}_{t} -> {dst}_{t} [label=\"{esc(label)}\", constraint=false, {style}];")

    for t, text in notes.items():
        L.append(f'  NOTE_{t} [label="{esc(text)}", shape=note, fontsize=8, '
                 f'style="rounded,filled", fillcolor="#fff3e0", color="#e65100"];')

    for t in range(1, nrows + 1):
        grp = [f"{c}_{t}" for c in cols]
        if t in notes:
            grp.append(f"NOTE_{t}")
        L.append(f"  {{ rank=same; {'; '.join(grp)}; }}")

    L.append("}")
    return "\n".join(L)


def render(name: str, src: str) -> None:
    dot = ROOT / f"{name}.dot"
    dot.write_text(src, encoding="utf-8")
    out = ROOT / f"{name}.png"
    subprocess.run(["dot", "-Tpng", str(dot), "-o", str(out)], check=True)
    print(f"{name}.png OK ({out.stat().st_size} B)")


# ── 1. Chemin d'écriture + fencing + propagation ────────────────────────────
seq_write = sequence(
    "seq_write",
    "Séquence — chemin d'écriture (fencing lease) puis propagation CRDT",
    [
        ("cli", "Client (A/B)", "#ffffff"),
        ("ora", "Orchestrateur A (leader)", "#ede7f6"),
        ("etd", "etcd « /amane/leader »", "#fff8e1"),
        ("orb", "Orchestrateur B (pair)", "#e3f2fd"),
    ],
    [
        ("cli", "ora", "Write(journal_id, site_id, payload)", "style=solid"),
        ("ora", "etd", "IsLeader() — lease détenue ?", "style=dashed, dir=forward, color=#b07c1a"),
        ("etd", "ora", "ok — leader (lease /amane/leader)", "style=dashed, color=#999, arrowsize=0.7"),
        ("ora", "ora", "journal.Append(...) → ack", "style=dashed, dir=forward, color=#2e7d32"),
        ("ora", "ora", "Relay.Add(inc, dec) — delta + seq → pending", "style=dashed, dir=forward, color=#5e35b1"),
        ("ora", "orb", "PushDelta {totaux cumulés, traceparent}", "style=solid, color=#c62828"),
        ("orb", "ora", "ack = dernière seq appliquée", "style=dashed, color=#999"),
        ("ora", "ora", "Confirm(ackSeq) — gc du pending", "style=dashed, dir=forward, color=#5e35b1"),
    ],
    notes={
        2: "si non-leader → codes.FailedPrecondition\n« nœud non leader (fencing lease) »\navant TOUT accès journal",
    },
)

# ── 2. Failover crash (superviseur) ─────────────────────────────────────────
seq_failover = sequence(
    "seq_failover",
    "Séquence — failover sur crash (superviseur, cible < 5 s)",
    [
        ("pri", "postgres-primary", "#e8f5e9"),
        ("sup", "Superviseur (réplica)", "#ede7f6"),
        ("etd", "etcd (heartbeats + lock)", "#fff8e1"),
        ("pat", "Patroni / candidat sync", "#e3f2fd"),
    ],
    [
        ("pri", "pri", "#crash# : SIGKILL (aucun nettoyage)", "style=dashed, dir=forward, color=#c62828"),
        ("etd", "sup", "heartbeat primary + REST en échec (500 ms)", "style=dashed, color=#999"),
        ("sup", "sup", "StaleConfirm = 2 ticks → crash confirmé", "style=dashed, dir=forward, color=#e65100"),
        ("sup", "etd", "DELETE /service/amane/leader (libération du lock)", "style=solid"),
        ("sup", "pat", "POST /failover (SANS le champ leader)", "style=solid, color=#c62828"),
        ("pat", "etd", "recapture du lock + promotion", "style=dashed, color=#999"),
        ("etd", "pri", "fencing : l'ancien primary ne recapture jamais", "style=dashed, color=#999, dir=forward"),
        ("pat", "sup", "ok — failover ≈ 3,2 s (détection 0,8 s)", "style=dashed, color=#2e7d32"),
    ],
    notes={
        4: "force=true seul insuffisant : sans la\nlibération du lock, la promotion resterait\nbornée par ttl >= 20 s",
    },
)

# ── 3. Réplication PushDelta / ack / Confirm ────────────────────────────────
seq_push = sequence(
    "seq_push",
    "Séquence — réplication multi-site PushDelta (garantie AP)",
    [
        ("rel1", "Relay A (atelier)", "#ede7f6"),
        ("pro1", "Propagateur A", "#e3f2fd"),
        ("srv2", "gRPC serveur B", "#e3f2fd"),
        ("rel2", "Relay B (eshop)", "#ede7f6"),
    ],
    [
        ("rel1", "pro1", "Outgoing() — deltas pending (1 s)", "style=dashed, dir=forward, color=#5e35b1"),
        ("pro1", "srv2", "PushDelta {site_id, node, inc/dec, seq}", "style=solid, color=#c62828"),
        ("srv2", "rel2", "Accept(fromNode, deltas)", "style=dashed, dir=forward, color=#5e35b1"),
        ("rel2", "rel2", "max-merge → valeur locale (total cumulé)", "style=dashed, dir=forward, color=#2e7d32"),
        ("srv2", "pro1", "ack {seq appliée, valeur}", "style=dashed, color=#999"),
        ("pro1", "rel1", "Confirm(ackSeq) — gc du pending", "style=dashed, dir=forward, color=#5e35b1"),
    ],
    notes={
        2: "échec réseau ? repoussé au tick suivant\n(jusqu'à ack) — les deltas ne sont jamais perdus\ndoublons/trous/réordonnancements inoffensifs (max)",
    },
)

# ── 4. Enrôlement / révocation (Interface 1 A↔C) ────────────────────────────
seq_enroll = sequence(
    "seq_enroll",
    "Séquence — enrôlement & révocation de machine (Interface 1, Mission A ↔ C)",
    [
        ("a", "Machine (Mission A)", "#ffffff"),
        ("orc", "Orchestrateur C (gRPC)", "#ede7f6"),
        ("reg", "etcd « /amane/members »", "#fff8e1"),
    ],
    [
        ("a", "orc", "Enroll {machine_id, AK publique, site, opérateur}", "style=solid"),
        ("orc", "reg", "membership.Add(...)", "style=dashed, dir=forward, color=#b07c1a"),
        ("reg", "orc", "quorum recalculé", "style=dashed, color=#999"),
        ("orc", "a", "EnrollResponse {status, quorum}  |  AlreadyExists si présent", "style=dashed, color=#2e7d32"),
        ("a", "orc", "NotifyRevocation {machine_id}", "style=solid, color=#c62828"),
        ("orc", "reg", "membership.Remove(...) → quorum", "style=dashed, dir=forward, color=#b07c1a"),
        ("reg", "orc", "machine retirée (plus jamais d'AK)", "style=dashed, color=#999"),
        ("orc", "a", "ack révocation", "style=dashed, color=#2e7d32"),
    ],
    notes={
        1: "jamais de clé en clair : seule la clé PUBLIQUE\nAK circule (règle transverse n°1)",
    },
)

for name, src in [
    ("seq_write", seq_write),
    ("seq_failover", seq_failover),
    ("seq_push", seq_push),
    ("seq_enroll", seq_enroll),
]:
    render(name, src)