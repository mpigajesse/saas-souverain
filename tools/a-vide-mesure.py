#!/usr/bin/env python3
"""A-04 — collecte de la consommation de la pile AMANE « à vide ».

Deux séries synchronisées, toutes les ``A04_STEP`` secondes pendant
``A04_SAMPLES`` points :
  * stats.csv : docker stats --no-stream, un ligne par conteneur de la pile ;
  * hote.csv  : /proc/loadavg, /proc/meminfo, /proc/diskstats, sensors -u.

Variables d'environnement (essais uniquement) : A04_OUT, A04_SAMPLES, A04_STEP.
Aucune modification de la pile : uniquement des lectures.
"""

import csv
import datetime as dt
import os
import re
import subprocess
import sys
import time

OUT = os.environ.get("A04_OUT", "docs/tests/a-vide")
SAMPLES = int(os.environ.get("A04_SAMPLES", "61"))
STEP = int(os.environ.get("A04_STEP", "10"))

PILE = [
    "amane-etcd",
    "amane-postgresql",
    "amane-orchestrateur",
    "amane-haproxy",
    "ss-node",
    "amane-application",
    "pgadmin",
]

STATS_FMT = (
    "{{.Name}}|{{.CPUPerc}}|{{.MemUsage}}|{{.MemPerc}}|"
    "{{.NetIO}}|{{.BlockIO}}|{{.PIDs}}"
)

STATS_HEADER = [
    "horodatage_utc", "epoch", "nom", "cpu_pct",
    "mem_brut", "mem_octets", "mem_limite_brut", "mem_pct",
    "reseau_entrant_brut", "reseau_entrant_octets",
    "reseau_sortant_brut", "reseau_sortant_octets",
    "bloc_lecture_brut", "bloc_lecture_octets",
    "bloc_ecriture_brut", "bloc_ecriture_octets", "pids",
]

HOTE_HEADER = [
    "horodatage_utc", "epoch", "charge_1", "charge_5", "charge_15",
    "mem_totale_kb", "mem_disponible_kb", "mem_libre_kb",
    "cpu_logiques", "processus_actifs", "processus_total",
    "disk_secteurs_lus", "disk_secteurs_ecrits",
    "disk_octets_lus", "disk_octets_ecrits",
    "disk_lectures", "disk_ecritures", "disk_ms_io",
    "temp_package_c", "temp_core_max_c", "temp_nvme0_c", "temp_dell_temp1_c",
]

UNITES = {
    "B": 1,
    "kB": 1000, "MB": 1000 ** 2, "GB": 1000 ** 3, "TB": 1000 ** 4,
    "KiB": 1024, "MiB": 1024 ** 2, "GiB": 1024 ** 3, "TiB": 1024 ** 4,
}


def octets(brut):
    """Convertit une taille affichée par docker stats en octets."""
    m = re.match(r"^\s*([0-9.]+)\s*([A-Za-z]*)\s*$", brut)
    if not m:
        return ""
    return str(int(float(m.group(1)) * UNITES.get(m.group(2), 1)))


def deux_valeurs(brut):
    """« 1.2MB / 3.4MB » -> (brut gauche, brut droite)."""
    parties = [p.strip() for p in brut.split("/")]
    if len(parties) != 2:
        return brut.strip(), "0B"
    return parties[0], parties[1]


def echantillon_stats(epoch, iso):
    ligne = [iso, str(epoch)]
    try:
        p = subprocess.run(
            ["docker", "stats", "--no-stream", "--format", STATS_FMT],
            capture_output=True, text=True, timeout=25, check=False,
        )
    except subprocess.TimeoutExpired:
        return []
    sorties = []
    for text in p.stdout.splitlines():
        champs = text.split("|")
        if len(champs) < 7:
            continue
        nom, cpu, mem, mempct, net, blk, pids = champs[:7]
        if nom not in PILE:
            continue
        mem_u, limite_u = deux_valeurs(mem)
        net_in, net_out = deux_valeurs(net)
        blk_in, blk_out = deux_valeurs(blk)
        sorties.append([
            iso, str(epoch), nom, cpu.replace("%", "").strip(),
            mem_u, octets(mem_u), limite_u, mempct.replace("%", "").strip(),
            net_in, octets(net_in), net_out, octets(net_out),
            blk_in, octets(blk_in), blk_out, octets(blk_out), pids.strip(),
        ])
    return sorties


def temperatures():
    """Extrait les sondes utiles de `sensors -u` en une seule lecture.

    Dans `sensors -u`, les libellés de sondes (« Package id 0: », « Core 0: »)
    ne sont PAS indentés ; seules les valeurs le sont. Un libellé se reconnaît
    donc au `:` final, une puce au fait de ne pas avoir de `:`.
    """
    resultat = {"pkg": "", "core": "", "nvme0": "", "dell": ""}
    try:
        p = subprocess.run(
            ["sensors", "-u"], capture_output=True, text=True,
            timeout=10, check=False,
        )
    except subprocess.TimeoutExpired:
        return resultat
    puce = None
    sonde = None
    coeurs = []
    for ligne in p.stdout.splitlines():
        texte = ligne.strip()
        if not texte:
            continue
        if not ligne.startswith(" "):
            if texte.startswith("Adapter:"):
                continue
            if texte.endswith(":"):
                sonde = texte
            else:
                puce = texte
                sonde = None
            continue
        m = re.match(r"^([a-zA-Z0-9_]+):\s*(-?[0-9.]+)", texte)
        if not m or sonde is None or puce is None:
            continue
        cle, valeur = m.group(1), m.group(2)
        if puce.startswith("coretemp") and sonde.startswith("Package id 0") and cle == "temp1_input":
            resultat["pkg"] = valeur
        elif puce.startswith("coretemp") and sonde.startswith("Core ") and cle.endswith("_input"):
            coeurs.append(float(valeur))
        elif puce == "nvme-pci-0300" and sonde == "Composite:" and cle == "temp1_input":
            resultat["nvme0"] = valeur
        elif puce == "dell_smm-virtual-0" and sonde == "temp1:" and cle == "temp1_input":
            resultat["dell"] = valeur
    if coeurs:
        resultat["core"] = str(max(coeurs))
    return resultat


def lire_cle_fichier(chemin, cle):
    with open(chemin, encoding="utf-8") as f:
        for ligne in f:
            if ligne.startswith(cle + ":"):
                return ligne.split(":", 1)[1].strip().split()[0]
    return ""


def echantillon_hote(epoch, iso, ncpu):
    charge = open("/proc/loadavg", encoding="utf-8").read().split()
    total_kb = lire_cle_fichier("/proc/meminfo", "MemTotal")
    dispo_kb = lire_cle_fichier("/proc/meminfo", "MemAvailable")
    libre_kb = lire_cle_fichier("/proc/meminfo", "MemFree")

    # Seuls les disques physiques entiers (pas les partitions, ni zram)
    # pour éviter de compter deux fois les mêmes octets.
    secteurs_lus = secteurs_ecrits = lectures = ecritures = ms_io = 0
    with open("/proc/diskstats", encoding="utf-8") as f:
        for ligne in f:
            champs = ligne.split()
            if len(champs) < 14:
                continue
            nom = champs[2]
            if not re.match(r"^nvme[0-9]+n[0-9]+$", nom):
                continue
            lectures += int(champs[3])
            secteurs_lus += int(champs[5])
            ecritures += int(champs[7])
            secteurs_ecrits += int(champs[9])
            ms_io += int(champs[12])

    temp = temperatures()
    return [
        iso, str(epoch), charge[0], charge[1], charge[2],
        total_kb, dispo_kb, libre_kb, str(ncpu),
        charge[3].split("/")[0], charge[3].split("/")[1],
        str(secteurs_lus), str(secteurs_ecrits),
        str(secteurs_lus * 512), str(secteurs_ecrits * 512),
        str(lectures), str(ecritures), str(ms_io),
        temp["pkg"], temp["core"], temp["nvme0"], temp["dell"],
    ]


def ecrire_csv(chemin, entete, lignes):
    with open(chemin, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(entete)
        w.writerows(lignes)


def main():
    os.makedirs(OUT, exist_ok=True)
    ncpu = os.cpu_count() or 0
    debut = int(time.time())
    cible = (debut // STEP + 1) * STEP

    meta = [
        "# A-04 — métadonnées de la fenêtre de mesure",
        f"debut_demande_epoch={debut}",
        f"debut_demande_utc={dt.datetime.fromtimestamp(debut, dt.timezone.utc).isoformat()}",
        f"pas_secondes={STEP}",
        f"nombre_points_prevus={SAMPLES}",
        f"duree_prevue_secondes={(SAMPLES - 1) * STEP}",
        f"cpu_logiques={ncpu}",
        "conteneurs_suivis=" + ",".join(PILE),
        "commande=python3 tools/a-vide-mesure.py",
        "host_cpu=/proc/loadavg ; host_ram=/proc/meminfo (MemAvailable) ; "
        "host_io=/proc/diskstats (nvme entiers, secteurs de 512 octets) ; "
        "host_temp=sensors -u",
        "contexte=3 VM QEMU amane-bench (4 Go chacune) laissées allumées, "
        "non sollicitées pendant la fenêtre ; aucune modification de la pile",
    ]

    stats, hote = [], []
    for rang in range(SAMPLES):
        maintenant = time.time()
        while maintenant < cible:
            time.sleep(min(1.0, cible - maintenant))
            maintenant = time.time()
        epoch = cible
        iso = dt.datetime.fromtimestamp(epoch, dt.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )
        hote.append(echantillon_hote(epoch, iso, ncpu))
        stats.extend(echantillon_stats(epoch, iso))
        cible += STEP
        if (rang + 1) % 10 == 0:
            print(f"point {rang + 1}/{SAMPLES} — {iso}", flush=True)

    fin = int(time.time())
    meta += [
        f"fin_reelle_epoch={fin}",
        f"fin_reelle_utc={dt.datetime.fromtimestamp(fin, dt.timezone.utc).isoformat()}",
        f"duree_reelle_secondes={fin - debut}",
        f"lignes_stats={len(stats)}",
        f"lignes_hote={len(hote)}",
    ]
    ecrire_csv(os.path.join(OUT, "stats.csv"), STATS_HEADER, stats)
    ecrire_csv(os.path.join(OUT, "hote.csv"), HOTE_HEADER, hote)
    with open(os.path.join(OUT, "meta.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(meta) + "\n")
    print("Fin : " + "\n".join(meta[-5:]), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
