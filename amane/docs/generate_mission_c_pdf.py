#!/usr/bin/env python
"""Génère docs/mission_c_architecture.pdf via ReportLab.

Lancement (depuis la racine du repo) :
    .venv/bin/python docs/generate_mission_c_pdf.py

Le contenu des sections « Contrat proto » et « Environnement local » est extrait
des sources réelles (proto/*.proto et docker-compose.yml) : le PDF reste
synchronisé avec le code. Les autres sections reprennent l'état d'avancement
réel des modules orchestrator-go/consensus, replication, mesh et grpcserver.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.platypus import (
    Paragraph,
    PageBreak,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)

REPO = Path(__file__).resolve().parent.parent
PROTO_FILE = REPO / "proto" / "amane" / "framework" / "v1" / "framework.proto"
COMPOSE_FILE = REPO / "docker-compose.yml"
OUT = Path(__file__).resolve().parent / "mission_c_architecture.pdf"

ACCENT = colors.HexColor("#1F3A5F")
LIGHT = colors.HexColor("#EEF2F7")
MID = colors.HexColor("#C9D4E1")


# ---------------------------------------------------------------------------
# Extraction depuis les sources réelles
# ---------------------------------------------------------------------------
def parse_proto(path: Path) -> dict:
    """Extrait service (rpc) et messages depuis framework.proto."""
    text = path.read_text(encoding="utf-8")
    service = []
    for m in re.finditer(r"//\s*(.*?)\n\s*rpc\s+(\w+)\((\w+)\)\s+returns\s+\((\w+)\);",
                         text, re.S):
        comment, rpc, req, resp = [g.strip() for g in m.groups()]
        service.append((rpc, req, resp, comment.split("\n")[-1]))
    messages = {}
    for m in re.finditer(r"message\s+(\w+)\s*\{([^}]*)\}", text, re.S):
        name, body = m.group(1), m.group(2)
        fields = []
        for f in re.finditer(r"\s*(repeated\s+)?([\w.]+)\s+(\w+)\s*=\s*(\d+);", body):
            repeated, ftype, fname, num = f.groups()
            fields.append((f"{'repeated ' if repeated else ''}{ftype}",
                           fname, int(num)))
        messages[name] = fields
    return {"service": service, "messages": messages}


def parse_compose(path: Path) -> list[tuple[str, str, str]]:
    """Extrait services docker-compose : (nom, image, ports)."""
    text = path.read_text(encoding="utf-8")
    services = []
    for block in re.split(r"\n  (?=\w+:)", text):
        name_m = re.match(r"  (\w+):", block)
        if not name_m:
            continue
        name = name_m.group(1)
        image = re.search(r"image:\s*(\S+)", block)
        ports = re.findall(r'-\s*"(\d+):\d+"', block)
        services.append((name, image.group(1) if image else "-",
                         ", ".join(ports) or "-"))
    return services


PROTO = parse_proto(PROTO_FILE)
COMPOSE = parse_compose(COMPOSE_FILE)

# ---------------------------------------------------------------------------
# Styles
# ---------------------------------------------------------------------------
S = getSampleStyleSheet()
title = ParagraphStyle("t", parent=S["Title"], fontSize=22, textColor=ACCENT)
h1 = ParagraphStyle("h1", parent=S["Heading1"], fontSize=15, textColor=ACCENT,
                    spaceBefore=14, spaceAfter=6)
h2 = ParagraphStyle("h2", parent=S["Heading2"], fontSize=12, textColor=ACCENT,
                    spaceBefore=10, spaceAfter=4)
body = ParagraphStyle("b", parent=S["BodyText"], fontSize=9.5, leading=13)
mono = ParagraphStyle("mo", parent=body, fontName="Courier", fontSize=8.5)
small = ParagraphStyle("s", parent=body, fontSize=8, leading=11, textColor=colors.grey)


def table(rows, col_widths, header=True):
    t = Table(rows, colWidths=col_widths, repeatRows=1 if header else 0)
    style = [
        ("GRID", (0, 0), (-1, -1), 0.4, MID),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("BACKGROUND", (0, 0), (-1, 0), LIGHT if header else colors.white),
        ("FONTSIZE", (0, 0), (-1, -1), 8),
        ("TOPPADDING", (0, 0), (-1, -1), 4),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
    ]
    if header:
        style.append(("FONTNAME", (0, 0), (-1, 0), "Courier-Bold"))
    t.setStyle(TableStyle(style))
    return t


def cell(value: str) -> str:
    return f"<font face='Courier' size='7.5'>{value}</font>"


# ---------------------------------------------------------------------------
# Document
# ---------------------------------------------------------------------------
story = []

# --- Couverture -------------------------------------------------------------
story.append(Spacer(1, 40 * mm))
story.append(Paragraph("AMANE — Mission C", title))
story.append(Paragraph("Infrastructure &amp; résilience du cluster distribué",
                       ParagraphStyle("sub", parent=body, fontSize=13,
                                      textColor=colors.grey)))
story.append(Spacer(1, 8 * mm))
story.append(Paragraph("orchestrator-go · consensus (etcd · Patroni) · réplication "
                       "CRDT · mesh WireGuard · serveur gRPC", small))
story.append(PageBreak())

# --- Vue d'ensemble ---------------------------------------------------------
story.append(Paragraph("1. Vue d'ensemble", h1))
story.append(Paragraph(
    "Mission C porte l'infrastructure et la résilience du cluster AMANE : "
    "consensus distribué (etcd Raft), haute disponibilité PostgreSQL (Patroni, "
    "réplication synchrone), réplication CRDT delta multi-site, réseau intra-site "
    "WireGuard et serveur gRPC exposant le framework aux Missions A et B.", body))
story.append(Paragraph("Jalons", h2))
story.append(table([
    ["Jalon", "Contenu", "État"],
    ["1", "Contrat proto B↔C, squelette gRPC (Ping, TLS, interceptors)", "Terminé"],
    ["2", "Consensus : membership, quorum, lease/fencing etcd, Enroll/NotifyRevocation", "Terminé"],
    ["3", "Réplication CRDT delta multi-site + Write/Read", "Terminé"],
    ["4", "Mesh WireGuard intra-site + failover < 5 s, zéro perte", "Terminé"],
], [20 * mm, 100 * mm, 35 * mm]))

# --- Contrat proto ------------------------------------------------------------
story.append(Paragraph("2. Contrat proto B ↔ C", h1))
story.append(Paragraph(
    f"Source : <font face='Courier' size='8'>proto/amane/framework/v1/framework.proto</font>. "
    "Ne jamais renuméroter un champ ni changer son type (compatibilité binaire, "
    "contrôlée par <font face='Courier' size='8'>buf lint</font> et "
    "<font face='Courier' size='8'>buf breaking</font> en CI).", body))
story.append(Paragraph("Services", h2))
story.append(table([["RPC", "Requête", "Réponse", "Description"]]
                  + [[cell(rpc), cell(req), cell(resp), desc] for rpc, req, resp, desc in PROTO["service"]],
                  [30 * mm, 30 * mm, 30 * mm, 65 * mm]))
story.append(Paragraph("Messages", h2))
for name, fields in PROTO["messages"].items():
    story.append(Paragraph(f"<font face='Helvetica-Bold' size='9'>{name}</font>", body))
    story.append(table([["Champ", "Type", "№"]]
                      + [[cell(fname), cell(ftype), cell(str(num))] for ftype, fname, num in fields],
                      [40 * mm, 90 * mm, 20 * mm]))

story.append(Paragraph("Verrouillage par les tests (module <font face='Courier' size='8'>tests/</font>)", h2))
story.append(Paragraph(
    "Module Go isolé <font face='Courier' size='8'>tests/</font> (package "
    "<font face='Courier' size='8'>tests/contracts</font>, go.mod "
    "<font face='Courier' size='8'>github.com/amane/tests</font>). "
    "<font face='Courier' size='8'>proto_contract_test.go</font> verrouille le contrat B ↔ C par "
    "les numéros de champs exacts de tous les messages partagés et la présence des 6 RPC — "
    "équivalent programme de <font face='Courier' size='8'>buf breaking</font> sans dépôt git "
    "(tourne toujours). <font face='Courier' size='8'>contracts_test.go</font> valide l'Interface 1 "
    "A ↔ C contre etcd réel (Enroll → registre, double-enroll → AlreadyExists, NotifyRevocation → "
    "quorum recalculé, révocation inconnue → NotFound) et le chemin d'écriture B ↔ C gated : "
    "Write (leader) → synced + Read retourne le payload + PushDelta site-b → site-a converge "
    "(value -2, acked_seq 1) ; Write non-leader → <font face='Courier' size='8'>"
    "codes.FailedPrecondition</font>, zéro entrée au journal. Lancement : "
    "<font face='Courier' size='8'>cd tests && AMANE_TEST_ETCD=localhost:2379 go test ./... "
    "-count=1 -race</font> (etcd réel requis, skip sinon).", body))

# --- Consensus ----------------------------------------------------------------
story.append(Paragraph("3. Consensus (etcd + Patroni)", h1))
story.append(Paragraph("Membership & quorum", h2))
story.append(Paragraph(
    "Le registre des membres vit dans etcd (clés <font face='Courier' size='8'>/members/{machine_id}</font>). "
    "Un membre est créé par transaction conditionnelle (<font face='Courier' size='8'>CreateRevision == 0</font>) "
    "pour garantir l'unicité. Le quorum suit la majorité Raft : "
    "<font face='Courier' size='8'>quorum = total/2 + 1</font>. Une révocation recalcule "
    "le quorum et le retourne via NotifyRevocation.", body))
story.append(Paragraph("Lease & fencing", h2))
story.append(Paragraph(
    "Le lead orchestrateur : TTL 30 s, renouvellement périodique, élection via "
    "<font face='Courier' size='8'>concurrency.Session + Election.Campaign</font>. "
    "Fencing : un nœud qui perd sa lease ne peut plus écrire ; reprise du leadership "
    "journalisée en JSON (log/slog) avec timing.", body))
story.append(Paragraph("Fencing applicatif — Write gated par lease", h2))
story.append(Paragraph(
    "<font face='Courier' size='8'>grpcserver.Server.Write</font> est refusé tant que le "
    "nœud ne détient pas la lease etcd (interface <font face='Courier' size='8'>"
    "grpcserver.Leadership</font>, câblée en prod via <font face='Courier' size='8'>"
    "consensus.NewLeadership</font>/<font face='Courier' size='8'>WithLeadership</font>) : "
    "<font face='Courier' size='8'>codes.FailedPrecondition</font> « write refusé : nœud non "
    "leader (fencing lease) » avant tout accès au journal (aucun compromis de séquence) — "
    "anti split-brain, l'élection etcd ne produit jamais deux leaders en même temps. Preuve "
    "live : 2 instances, Write sur le non-leader → FailedPrecondition puis commité (node_id, "
    "synced=true) après libération de la lease ; intégration "
    "<font face='Courier' size='8'>TestLeadershipFencingAgainstEtcd</font> "
    "(<font face='Courier' size='8'>AMANE_TEST_ETCD</font>). Relectures (Read, Ping, Enroll, "
    "NotifyRevocation) non gated ; sans <font face='Courier' size='8'>WithLeadership</font> le "
    "gating est désactivé (tests/rétro-compat), jamais en prod.", body))
story.append(table([
    ["Composant", "Détail"],
    ["quorum.go", "ComputeQuorum(total) = majorité stricte"],
    ["membership.go", "Registry Add/Remove/Has/Quorum (txn etcd)"],
    ["lease.go", "Leadership : Session TTL 30 s, Election, fencing"],
    ["client.go", "NewClient(endpoints) client etcd v3"],
], [50 * mm, 105 * mm]))

story.append(Paragraph("HA mesurée (protocole reproductible)", h2))
story.append(Paragraph(
    "Scripts : <font face='Courier' size='8'>scripts/failover_measure.sh crash|partition</font> "
    "et <font face='Courier' size='8'>scripts/switchover_measure.sh</font>. La mesure est "
    "instrumentée (horodatage T0, watch etcd horodatée, sondage REST 50 ms) et vérifie "
    "le fencing (ancien primary = réplica, pg_is_in_recovery=t) et le zéro perte "
    "(marqueur acké côté standby sync présent sur le nouveau primary).", body))
story.append(table([
    ["Scénario", "failover_ms", "writable_ms", "cible", "fencing", "zéro perte"],
    ["Switchover contrôlé (API Patroni)", "≈ 2 200", "≈ 2 400", "< 5 s ATTEINTE", "OK", "OK"],
    ["Crash (docker kill primary) — avec superviseur actif", "≈ 3 200", "≈ 4 000", "< 5 s ATTEINTE", "OK", "OK"],
    ["Partition réseau (docker network disconnect) — avec superviseur actif", "≈ 3 000", "≈ 3 400", "< 5 s ATTEINTE", "OK", "OK"],
], [56 * mm, 29 * mm, 25 * mm, 29 * mm, 16 * mm, 16 * mm]))
story.append(Paragraph(
    "<b>Lecture honnête :</b> le plancher de lease est une validation <b>Patroni</b> "
    "(<font face='Courier' size='8'>patroni/config.py</font>, \"can't be smaller than 20, "
    "adjusting\") appliquée au paramètre <font face='Courier' size='8'>ttl</font>, "
    "indépendamment du backend DCS — <b>changer de DCS (ZooKeeper, Kubernetes) ne lève pas "
    "le plancher</b>. Sans superviseur, la détection d'un crash non contrôlé est donc bornée "
    "à ≈ 21 s ; avec le superviseur actif, la libération du lock <font face='Courier' size='8'>"
    "/service/&lt;scope&gt;/leader</font> avant expiration fait passer le crash sous la "
    "cible < 5 s (mesuré ci-dessus) ; le chemin <b>contrôlé</b> (switchover planifié) reste "
    "déjà sous 5 s via le release volontaire de la lease. Le choix architectural entre les "
    "3 options est comparé ci-dessous.", small))

story.append(Paragraph("Décision d'architecture HA — 3 options pour un crash &lt; 5 s", h2))
story.append(Paragraph(
    "Le lock Patroni est anti split-brain : personne ne capture le lock avant expiration "
    "de la lease de l'ancien primary. Les options B et C cherchent à réduire CE délai ; "
    "une 4e piste (changer de DCS) est écartée — prouvé ci-dessus, le plancher ttl est "
    "porté par Patroni, pas par etcd/zk/k8s.", body))
story.append(table([
    ["Critère", "A — Chemin contrôlé assumé",
                 "B — Baisser le plancher ttl",
                 "C — Orchestrateur externe"],
    ["Détection crash (SIGKILL)", "≈ 20 s (lease)", "≈ 3-5 s (ttl court)", "≈ 0,5-1 s (sondes actives)"],
    ["Bascule totale (crash)", "≈ 21 s", "≈ 4-6 s", "≈ 1-3 s"],
    ["Switchover planifié", "≈ 2,2 s (prouvé)", "≈ 2 s", "≈ 2 s"],
    ["Risque split-brain / faux failover", "Nul (plancher intact)", "Élevé (à-coups réseau ⇒ faux failover, split-brain masqué)", "Contrôlé (vérif quorum avant démotion forcée)"],
    ["Garantie zéro perte", "OK (répl. synchrone)", "OK mais période risquée", "OK (répl. synchrone)"],
    ["Fencing", "OK (Patroni)", "OK mais fenêtre fragilisée", "OK (démotion + kill)"],
    ["Effort Mission C", "Aucun (déjà prouvé)", "Moyen (patch config.py / override, re-tests, stress réseau)", "Élevé (nouveau composant, sa propre HA)"],
    ["Dépendances", "Aucune", "Fork/maintenance Patroni", "Composant superviseur + REST Patroni"],
    ["Cible < 5 s crash", "NON (assumée ~20 s)", "Étroitement, fragile", "OUI si composant HA"],
], [28 * mm, 40 * mm, 46 * mm, 44 * mm]))
story.append(table([
    ["Choix retenu (v1)", "A — chemin contrôlé assumé (switchover planifié < 5 s) + C — Superviseur "
                          "maison (Go, orchestrator-go) : crash ≈ 3,2 s avec superviseur actif (mesuré), "
                          "sans superviseur borné par la lease ≈ 21 s. B écarté (affaiblit la garantie "
                          "anti split-brain)."],
], [158 * mm, 0]))
story.append(Spacer(1, 3 * mm))

story.append(Paragraph("Superviseur maison — option C retenue (décision d'architecture)", h2))
story.append(Paragraph(
    "Le crash non contrôlé passe par un <b>superviseur dans orchestrator-go</b> "
    "(package <font face='Courier' size='8'>supervisor/</font> : config.go, evaluator.go, "
    "beacon.go, patroni.go, supervisor.go), déployé sur les 3 nœuds. <b>Pourquoi ce choix :</b> "
    "(1) mono-repo Go, aucune nouvelle infrastructure (pas de Kubernetes, pas de Pacemaker) ; "
    "(2) etcd déjà présent — heartbeat <b>propre</b> du superviseur à lease courte (2 s), "
    "<b>indépendante</b> du plancher <font face='Courier' size='8'>ttl &gt;= 20 s</font> de "
    "Patroni ; (3) pas de fork de Patroni, chemin de promotion déjà éprouvé à ~2,2 s en "
    "switchover ; (4) le superviseur <b>ne décide pas la promotion</b> : Patroni reste l'autorité "
    "de fencing et de promotion (réplique synchrone et zéro perte préservées) — simple "
    "déclencheur, pas une couche HA superposée ; (5) HA du superviseur : élection via un lock "
    "etcd « droit de forcer » (une seule instance agit), jamais de SPOF. <b>Mécanisme final :</b> "
    "le superviseur (1) publie dans etcd un <b>heartbeat propre à lease courte (2 s)</b> "
    "rapportant le rôle local ; (2) détecte le crash — probe REST du primary en échec <b>et</b> "
    "heartbeat etcd stale/absent après StaleConfirm=2 ticks — avec garde anti-partition : "
    "(heartbeat frais ⇒ primary vivant mais isolé ⇒ pas de forçage — split-brain partiel) ; "
    "en <b>coupure complète</b> (partition réseau), le heartbeat stoppe ⇒ forçage légitime "
    "(confirmé par la mesure : 3,0 s) ; (3) <b>libère le lock "
    "Patroni</b> en supprimant <font face='Courier' size='8'>/service/&lt;scope&gt;/leader</font> "
    "dans etcd (fencing conditionné : autorisé seulement quand le heartbeat du nœud est confirmé "
    "mort — l'ancien primary ne peut plus se battre) ; (4) appelle <font face='Courier' "
    "size='8'>POST /failover</font> <b>sans le champ <font face='Courier' size='8'>leader</font>"
    "</b> — avec <font face='Courier' size='8'>leader</font>, Patroni le traite comme switchover "
    "et attend un ancien joignable — et répond quand la promotion est finie (Patroni promeut en "
    "~2-3 s une fois le lock libéré). <b><font face='Courier' size='8'>force=true</font> seul ne "
    "suffisait pas</b> : sans libération du lock, la promotion restait bornée par l'expiration de "
    "la lease (ttl &gt;= 20 s). Timing par défaut : ProbeInterval 500 ms, HeartbeatTTL 2 s, "
    "StaleConfirm 2, LockTTL 3 s, CoolDown 30 s.", body))
story.append(Paragraph(
    "<b>Mesuré (stack réelle) : crash 3,2 s failover / 4,0 s writable ; partition (coupure complète, "
    "docker network disconnect) 3,0 s failover / 3,4 s writable — fencing + zéro perte OK sur les "
    "deux chemins.</b>", body))
story.append(table([
    ["Élément", "Design"],
    ["Détection", "probe REST Patroni (500 ms) + heartbeat etcd propre (lease 2 s) ; crash si probe en échec ET heartbeat stale/absent (StaleConfirm=2 ticks)"],
    ["Déclenchement", "libération lock etcd /service/<scope>/leader + POST /failover sans leader — Patroni répond 200 à la fin de promotion (~2-3 s)"],
    ["Fencing", "conditionné au heartbeat stale (nœud confirmé mort) — l'ancien primary ne recapture pas le lock ; Patroni reste l'autorité de promotion"],
    ["Autorisation", "lock etcd « droit de forcer » : une seule instance agit (élection ~ms)"],
    ["Cas exclus", "partition réseau / primary vivant : heartbeat frais ⇒ pas de forçage — Patroni tranche ; force=true seul insuffisant (sinon borné ttl >= 20 s)"],
], [35 * mm, 123 * mm]))

# --- Réplication ----------------------------------------------------------------
story.append(Paragraph("4. Réplication CRDT (delta multi-site)", h1))
story.append(Paragraph(
    "<b>Jalon 3 — terminé.</b> Delta-CRDT / PN-Counter pour les quantités de stock : "
    "<font face='Courier' size='8'>replication/counter.go</font> (Apply, Merge "
    "max-par-nœud, Value), convergence déterministe quel que soit l'ordre des deltas "
    "(tests 3 sites, 0 conflit non résolu). Journal multi-site "
    "<font face='Courier' size='8'>replication/journal.go</font> : écriture append-only, "
    "séquences monotones, rejeu idempotent (même site_id + op_seq → même seq committé), "
    "pagination (limit ≤ 1000). Write/Read branchés sur le gRPC (Interface 3 B ↔ C) ; "
    "les payloads restent chiffrés (encrypted_payload de Mission A, jamais de clair côté "
    "C). <b>Jalon 4 — relay multi-site implémenté</b> (voir ci-dessous).", body))
story.append(Paragraph(
    "<b>Relay multi-site — <font face='Courier' size='8'>replication/relay.go</font> "
    "(jalon 4, garantie AP).</b> Un delta porte le <b>total cumulé</b> du nœud émetteur "
    "(Inc/Dec depuis son origine, pas un incrément) → fusion <b>max par nœud émetteur</b> : "
    "commutative, associative, idempotente — doublons, réordonnancements et trous de "
    "séquence ne compromettent jamais la convergence. <font face='Courier' size='8'>"
    "Add</font> met à jour total + pending + seq ; <font face='Courier' size='8'>Accept"
    "</font> fusionne côté récepteur (réflexion d'un delta auto-émis ignorée) ; "
    "<font face='Courier' size='8'>Outgoing</font>/<font face='Courier' size='8'>"
    "Confirm(ackSeq)</font> font le gc du pending (fuite de mémoire évitée). RPC "
    "<b>additif</b> <font face='Courier' size='8'>PushDelta</font> au contrat B ↔ C "
    "(numéros de champs stables ; code généré committé, <font face='Courier' size='8'>"
    "buf generate</font> OK) ; handler <font face='Courier' size='8'>grpcserver</font> "
    "via <font face='Courier' size='8'>WithRelay(*replication.Relay)</font> — sans relay → "
    "<font face='Courier' size='8'>codes.Unavailable</font> ; câblé au démarrage dans "
    "<font face='Courier' size='8'>cmd/orchestrator/main.go</font>. <b>Preuve live "
    "(grpcurl)</b> : site-eshop inc 5 puis 7 → ack seq 2, value 7 ; puis site-atelier "
    "dec 3 → value 4. Tests : <font face='Courier' size='8'>relay_test.go</font> "
    "(convergence 3 nœuds réordonnés/doublés, ack gc du pending, réflexion ignorée, trou "
    "de seq idempotent, merge associatif) + <font face='Courier' size='8'>"
    "TestPushDeltaAcrossSites</font>/<font face='Courier' size='8'>TestPushDeltaWithoutRelay"
    "</font> (bufconn, 2 sites) — tout vert, y compris <font face='Courier' size='8'>-race"
    "</font>.", body))

# --- Mesh ----------------------------------------------------------------------
story.append(Paragraph("5. Mesh WireGuard (intra-site)", h1))
story.append(Paragraph(
    "<b>Jalon 4 — implémenté.</b> Package <font face='Courier' size='8'>mesh/</font> "
    "(orchestrator-go) : génération de la config <font face='Courier' size='8'>wg0.conf</font> "
    "par nœud — IP virtuelle stable par machine (10.10.&lt;site&gt;.&lt;n&gt;/24, inchangée au "
    "failover), <font face='Courier' size='8'>AllowedIPs = /32</font> du pair (jamais de "
    "routage global 0.0.0.0/0 rejeté par validation), <font face='Courier' size='8'>"
    "PersistentKeepalive = 25</font> (NAT/CGNAT), fichier .conf écrit en 0600 (clé privée "
    "jamais loggée). Usage strictement intra-site (etcd, Patroni, WAL) — l'inter-site "
    "passe par TLS sortant, jamais par WireGuard.", body))

# --- gRPC ----------------------------------------------------------------------
story.append(Paragraph("6. Serveur gRPC", h1))
story.append(table([
    ["Aspect", "Choix"],
    ["Transport", "gRPC/TLS 1.3 minimum (MinVersion=tls.VersionTLS13), mTLS si -tls-ca fourni (RequireAndVerifyClientCert)"],
    ["Chargement credentials", "server+client (client_credentials.go), CA interne, jamais InsecureSkipVerify"],
    ["Erreurs", "status.Error(codes.X, ...) — jamais d'erreur brute"],
    ["Intercepteurs", "Logging (slog JSON) + Recovery"],
    ["Introspection", "grpc reflection (grpcurl, dev)"],
    ["Lancement", ".venv non requis — go run ./cmd/orchestrator"],
], [55 * mm, 100 * mm]))
story.append(Paragraph(
    "mTLS prouvé e2e (TCP réel, PKI générée en mémoire) : client valide OK, "
    "refus sans cert, refus cert d'une autre CA, refus d'un handshake TLS 1.2. "
    "Vérif manuelle : grpcurl -cacert/-cert/-key + openssl s_client -tls1_2 refusé.", body))

# --- Environnement local ---------------------------------------------------------
story.append(Paragraph("7. Environnement local (docker-compose)", h1))
story.append(Paragraph(
    "etcd (Raft, source de vérité Patroni) + deux nœuds Spilo/PostgreSQL 17 en "
    "réplication synchrone. <font face='Courier' size='8'>synchronous_commit=on</font>, "
    "<font face='Courier' size='8'>max_slot_wal_keep_size=1024MB</font>.", body))
story.append(table([["Service", "Image", "Ports(hôte)"]]
                  + [[cell(n), cell(im), cell(p)] for n, im, p in COMPOSE],
                  [40 * mm, 85 * mm, 35 * mm]))
story.append(Paragraph(
    "Vérification Patroni : <font face='Courier' size='8'>curl -s localhost:8008/patroni</font> "
    "(primary/replica, sync_state).", small))

# --- Commandes -------------------------------------------------------------------
story.append(Paragraph("8. Commandes de développement", h1))
story.append(table([
    ["Commande", "Effet"],
    ["go build ./... / go test ./... -count=1", "Build & tests (depuis orchestrator-go/)"],
    ["docker compose up -d / down -v", "Stack etcd + Patroni (up / reset volumes)"],
    ["buf generate / lint / breaking", "Génération & vérifs proto (racine)"],
    ["~/go/bin/grpcurl -plaintext -d '{}' localhost:50051 .../Ping", "Test manuel gRPC"],
    ["~/go/bin/grpcurl -cacert tls/ca.crt -cert tls/client.crt -key tls/client.key -d '{}' 127.0.0.1:50051 .../Ping",
     "Test manuel mTLS (Ping)"],
    ["go test ./grpcserver/ -run TestCredentials -v", "Tests e2e TLS 1.3 / mTLS (PKI en mémoire)"],
    ["~/go/bin/grpcurl -plaintext -d '{\"journal_id\":\"stock\",...}' .../Write",
     "Écriture (Interface 3, rejeu idempotent)"],
    ["AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run Membership -v",
     "Intégration membership vs etcd réel"],
    ["AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run TestLeadershipFencing -v",
     "Élection + fencing lease-based (Write gated) vs etcd réel"],
    ["cd tests && AMANE_TEST_ETCD=localhost:2379 go test ./... -count=1 -race",
     "Tests cross-mission : contrats B↔C / A↔C + chemin d'écriture (etcd réel requis)"],
    ["bash scripts/failover_measure.sh crash|partition", "Mesure failover (fencing + zéro perte)"],
    ["bash scripts/switchover_measure.sh", "Mesure switchover contrôlé (< 5 s)"],
    [".venv/bin/python docs/generate_mission_c_pdf.py", "Régénère ce PDF (ReportLab)"],
], [95 * mm, 60 * mm]))

# --- Règles transverses -----------------------------------------------------------
story.append(Paragraph("9. Règles transverses (non négociables)", h1))
for rule in [
    "Jamais de clé en clair : AK privée, DEK, KEK interdits en Go, logs, env ou messages.",
    "Réplication synchrone : synchronous_commit=on, max_slot_wal_keep_size borné, failover < 5 s, zéro perte.",
    "Logs structurés log/slog (JSON), corrélation de timing failover/lease.",
    "gRPC : status.Error(codes.X, ...) — jamais d'erreur brute, ni nil, nil.",
    "Contrat proto : pas de renumérotation/changement de type ; code généré commité ; buf breaking en CI.",
]:
    story.append(Paragraph(f"• {rule}", body))


def build() -> None:
    doc = SimpleDocTemplate(str(OUT), pagesize=A4, leftMargin=18 * mm,
                            rightMargin=18 * mm, topMargin=15 * mm,
                            bottomMargin=15 * mm, title="AMANE — Mission C",
                            author="Mission C / docs-keeper",
                            subject="Infrastructure & résilience du cluster")
    doc.build(story)
    print(f"PDF généré : {OUT}")


if __name__ == "__main__":
    if not PROTO_FILE.exists() or not COMPOSE_FILE.exists():
        print(f"ERREUR : fichiers sources introuvables ({PROTO_FILE}, {COMPOSE_FILE})",
              file=sys.stderr)
        sys.exit(1)
    build()