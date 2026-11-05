# Démo publique du SaaS éditeur (gratuite, Render)

Seul le **SaaS éditeur** (Django : comptes, licences, parc, clusters) est mis en ligne. Le relais
zero-knowledge et les nœuds PME ne le sont pas : la page *Relais* l'indique « injoignable ».

Base **SQLite éphémère** : à chaque réveil du service gratuit, `migrate` puis
`seed_demo_public` recréent quatre PME fictives (licences, machines, une PME hors licence) et
deux comptes, `editeur` et `pme`, dont le mot de passe est `DEMO_PASSWORD`. Celui-ci est
**affiché** sur la page de connexion. En démo, l'administration Django (`/admin/`) est retirée.

Render → *New* → *Web Service* → ce dépôt, branche `demo-publique` :

| Champ | Valeur |
|---|---|
| Runtime | Python 3 |
| Build Command | `pip install -r requirements.txt && python manage.py collectstatic --noinput` |
| Start Command | `python manage.py migrate --noinput && python manage.py seed_demo_public && gunicorn config.wsgi:application --bind 0.0.0.0:$PORT --workers 2 --timeout 60` |
| Instance Type | Free |

Variables : `PYTHON_VERSION=3.12.7`, `DEBUG=False`, `SECRET_KEY` (*Generate*),
`DATABASE_URL=sqlite:///demo.sqlite3`, `DEMO_PASSWORD=<affiché publiquement>`,
`CSRF_TRUSTED_ORIGINS=https://<domaine personnalisé>` si un domaine est ajouté.
