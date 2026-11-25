#!/usr/bin/env python3
"""A-04 — synthèse des séries brutes collectées par a-vide-mesure.py.

Produit dans docs/tests/a-vide/ :
  * resume-conteneurs.csv : min/moyenne/max par conteneur + débits calculés ;
  * resume-hote.txt, evolution.txt : statistiques hôte et évolution par minute.
Les compteurs réseau/bloc de docker stats étant cumulatifs depuis le
démarrage du conteneur, les débits sont calculés par delta sur la fenêtre.
"""

import csv
import os
import statistics as st

OUT = os.environ.get("A04_OUT", "docs/tests/a-vide")
MIO = 1024 ** 2
GIO = 1024 ** 3


def lire_csv(nom):
    with open(os.path.join(OUT, nom), encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f))


def fl(d, cle):
    v = d.get(cle, "")
    return float(v) if v not in ("", None) else 0.0


def stats3(valeurs):
    return min(valeurs), st.fmean(valeurs), max(valeurs)


def fmt3(triplet, unite="", digits=2):
    a, b, c = triplet
    return f"{a:.{digits}f}{unite} / {b:.{digits}f}{unite} / {c:.{digits}f}{unite}"


def debit(rows, cle):
    """Débit moyen sur la fenêtre, en octets/s ; None si compteur remis à zéro."""
    if len(rows) < 2:
        return 0.0
    premier, dernier = rows[0], rows[-1]
    duree = int(dernier["epoch"]) - int(premier["epoch"])
    if duree <= 0:
        return 0.0
    delta = int(dernier[cle]) - int(premier[cle])
    if delta < 0:
        return None
    return delta / duree


def main():
    stats = lire_csv("stats.csv")
    hote = lire_csv("hote.csv")
    par_nom = {}
    for r in stats:
        par_nom.setdefault(r["nom"], []).append(r)

    resume = []
    print("=== CONTENEURS (min / moyenne / max) ===")
    print(f"{'conteneur':22} {'n':>3} {'CPU %':>26} {'Mémoire MiB':>26} {'PIDs':>14}")
    for nom, rows in sorted(par_nom.items()):
        cpu = stats3([fl(r, "cpu_pct") for r in rows])
        mem = stats3([fl(r, "mem_octets") / MIO for r in rows])
        pids = stats3([fl(r, "pids") for r in rows])
        rin = debit(rows, "reseau_entrant_octets")
        rout = debit(rows, "reseau_sortant_octets")
        bin_ = debit(rows, "bloc_lecture_octets")
        bout = debit(rows, "bloc_ecriture_octets")
        print(f"{nom:22} {len(rows):3} {fmt3(cpu):>26} {fmt3(mem):>26} "
              f"{fmt3(pids, digits=0):>14}")
        resume.append({
            "nom": nom, "points": len(rows),
            "cpu_min": f"{cpu[0]:.2f}", "cpu_moy": f"{cpu[1]:.2f}", "cpu_max": f"{cpu[2]:.2f}",
            "mem_mib_min": f"{mem[0]:.2f}", "mem_mib_moy": f"{mem[1]:.2f}", "mem_mib_max": f"{mem[2]:.2f}",
            "pids_moy": f"{pids[1]:.1f}",
            "net_entier_ko_s": "" if rin is None else f"{rin/1000:.3f}",
            "net_sortant_ko_s": "" if rout is None else f"{rout/1000:.3f}",
            "bloc_lecture_ko_s": "" if bin_ is None else f"{bin_/1000:.3f}",
            "bloc_ecriture_ko_s": "" if bout is None else f"{bout/1000:.3f}",
        })
    with open(os.path.join(OUT, "resume-conteneurs.csv"), "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(resume[0].keys()))
        w.writeheader()
        w.writerows(resume)

    mem_totale = [sum(fl(r, "mem_octets") for r in stats if r["epoch"] == e)
                  for e in dict.fromkeys(r["epoch"] for r in stats)]
    cpu_total = [sum(fl(r, "cpu_pct") for r in stats if r["epoch"] == e)
                 for e in dict.fromkeys(r["epoch"] for r in stats)]
    print("\n=== PILE TOTALE ===")
    print("mémoire MiB min/moy/max :", fmt3(stats3([m / MIO for m in mem_totale])))
    print("CPU % (somme) min/moy/max :", fmt3(stats3(cpu_total)))
    print("mémoire moyenne GiB :", f"{st.fmean(mem_totale)/GIO:.3f}")

    print("\n=== HÔTE ===")
    for cle, unite, div in [
        ("charge_1", "", 1), ("charge_5", "", 1), ("charge_15", "", 1),
        ("mem_disponible_kb", " GiB", 1024 ** 2),
        ("temp_package_c", " °C", 1), ("temp_core_max_c", " °C", 1),
        ("temp_nvme0_c", " °C", 1), ("temp_dell_temp1_c", " °C", 1),
    ]:
        v = stats3([fl(r, cle) / div for r in hote])
        print(f"{cle:22} {fmt3(v, unite)}")
    duree = int(hote[-1]["epoch"]) - int(hote[0]["epoch"])
    for cle, libelle in [("disk_octets_lus", "lecture"), ("disk_octets_ecrits", "écriture")]:
        d = (int(hote[-1][cle]) - int(hote[0][cle])) / duree / 1000 ** 2
        print(f"disque {libelle:10} : {d:.3f} Mo/s sur {duree} s")
    busy = (int(hote[-1]["disk_ms_io"]) - int(hote[0]["disk_ms_io"])) / (duree * 10)
    print(f"disque occupé (2 disques) : {busy:.2f} %")

    with open(os.path.join(OUT, "resume-hote.txt"), "w", encoding="utf-8") as f:
        f.write(f"fenêtre : {hote[0]['horodatage_utc']} → {hote[-1]['horodatage_utc']} "
                f"({duree} s)\n")
        for cle in ["charge_1", "charge_5", "charge_15", "mem_disponible_kb",
                    "temp_package_c", "temp_core_max_c", "temp_nvme0_c", "temp_dell_temp1_c"]:
            f.write(f"{cle} : {fmt3(stats3([fl(r, cle) for r in hote]))}\n")
        f.write(f"disk_lecture_mo_s : {(int(hote[-1]['disk_octets_lus'])-int(hote[0]['disk_octets_lus']))/duree/1000**2:.3f}\n")
        f.write(f"disk_ecriture_mo_s : {(int(hote[-1]['disk_octets_ecrits'])-int(hote[0]['disk_octets_ecrits']))/duree/1000**2:.3f}\n")
        f.write(f"disk_occupation_pct : {busy:.2f}\n")

    # Évolution par minute : 10 buckets de 6 points.
    print("\n=== ÉVOLUTION PAR MINUTE ===")
    print("minute UTC        mem_MiB  cpu_%  etcd_cpu  load1  dispo_GiB  lect_Mo/s  ecr_Mo/s  pkg_C")
    buckets = []
    for i in range(0, len(hote), 6):
        h = hote[i:i + 6]
        e0, e1 = int(h[0]["epoch"]), int(h[-1]["epoch"])
        span = max(e1 - e0, 1)
        mem_b = st.fmean(sum(fl(r, "mem_octets") for r in stats if r["epoch"] == row["epoch"])
                         for row in h) / MIO
        cpu_b = st.fmean(sum(fl(r, "cpu_pct") for r in stats if r["epoch"] == row["epoch"])
                         for row in h)
        etcd_b = st.fmean([fl(r, "cpu_pct") for r in stats
                           if r["nom"] == "amane-etcd" and int(r["epoch"]) >= e0 and int(r["epoch"]) <= e1])
        load_b = st.fmean([fl(r, "charge_1") for r in h])
        dispo_b = st.fmean([fl(r, "mem_disponible_kb") for r in h]) / 1024 ** 2
        lect_b = (int(h[-1]["disk_octets_lus"]) - int(h[0]["disk_octets_lus"])) / span / 1000 ** 2
        ecr_b = (int(h[-1]["disk_octets_ecrits"]) - int(h[0]["disk_octets_ecrits"])) / span / 1000 ** 2
        pkg_b = st.fmean([fl(r, "temp_package_c") for r in h])
        ligne = (f"{h[0]['horodatage_utc'][11:16]}      {mem_b:7.1f}  {cpu_b:5.2f} "
                 f"  {etcd_b:8.2f}  {load_b:5.2f}  {dispo_b:9.2f}  {lect_b:9.2f}  "
                 f"{ecr_b:8.2f}  {pkg_b:6.1f}")
        buckets.append(ligne)
        print(ligne)
    with open(os.path.join(OUT, "evolution.txt"), "w", encoding="utf-8") as f:
        f.write("minute UTC        mem_MiB  cpu_%  etcd_cpu  load1  dispo_GiB  lect_Mo/s  ecr_Mo/s  pkg_C\n")
        f.write("\n".join(buckets) + "\n")


if __name__ == "__main__":
    main()
