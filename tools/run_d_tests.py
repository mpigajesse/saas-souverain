#!/usr/bin/env python3
"""Lanceur du groupe D — tests du portail éditeur sur base SQLite jetable.

Usage (depuis la racine du dépôt) :
    python3 tools/run_d_tests.py            # tous les tests D-01 → D-06
    python3 tools/run_d_tests.py D01 D03    # une sélection de classes

La base de test est un fichier SQLite sous /tmp/opencode, créé puis détruit par
Django. La base PostgreSQL de production n'est jamais ouverte, la pile `amane-*`
n'est jamais touchée.
"""
from __future__ import annotations

import os
import sys
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(RACINE))
sys.path.insert(0, str(RACINE / 'tools'))

os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'dtests.settings_test')

# Dossier de la base jetable.
os.makedirs('/tmp/opencode/d-tests', exist_ok=True)

import django  # noqa: E402

django.setup()

from django.conf import settings  # noqa: E402
from django.test.utils import get_runner  # noqa: E402

# Tous les codes du groupe, dans l'ordre.
LABELS = {
    'D01': 'dtests.tests_portail.D01UsurpationTests',
    'D02': 'dtests.tests_portail.D02LicenceExpireeTests',
    'D03': 'dtests.tests_portail.D03SiegesConcurrentsTests',
    'D04': 'dtests.tests_portail.D04ConfinementComposeTests',
    'D05': 'dtests.tests_portail.D05ManifesteDangereuxTests',
    'D06': 'dtests.tests_portail.D06RegistreTests',
}


def main(argv: list[str]) -> int:
    selection = [a for a in argv if a in LABELS]
    labels = [LABELS[c] for c in selection] if selection else list(LABELS.values())
    print(f"Base de test : {settings.DATABASES['default']['TEST']['NAME']}")
    print(f"Lancement : {', '.join(labels)}")
    runner = get_runner(settings)(verbosity=2, interactive=False, keepdb=False)
    echecs = runner.run_tests(labels)
    print(f"\n=== {echecs} échec(s)/erreur(s) ===")
    return 1 if echecs else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))