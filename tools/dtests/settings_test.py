"""Réglages de test du groupe D — base SQLite **jetable**.

On importe les réglages réels du projet (`config.settings`, qui lit `.env` via
python-decouple) puis on remplace la base par une base SQLite de test, créée et
détruite par le lanceur de tests Django. La base de production PostgreSQL n'est
jamais ouverte : seul le moteur SQLite est utilisé, dans un fichier jetable sous
`/tmp/opencode`.
"""
from config.settings import *  # noqa: F401,F403

# Base de test jetable : fichier SQLite sous /tmp/opencode, recréée à chaque
# exécution. `timeout` (busy timeout) laisse les écritures concurrentes du test
# D-03 se sérialiser proprement plutôt que d'échouer en « database is locked ».
DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.sqlite3',
        'NAME': '/tmp/opencode/d-tests/amane-d.sqlite3',
        'OPTIONS': {'timeout': 30},
        'TEST': {'NAME': '/tmp/opencode/d-tests/test-amane-d.sqlite3'},
    }
}

# Hachage rapide : les tests ne créent pas de mots de passe de production.
PASSWORD_HASHERS = ['django.contrib.auth.hashers.MD5PasswordHasher']