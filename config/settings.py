from pathlib import Path
from decouple import config

BASE_DIR = Path(__file__).resolve().parent.parent

SECRET_KEY = config('SECRET_KEY')
DEBUG = config('DEBUG', default=False, cast=bool)

# ALLOWED_HOSTS est défini plus bas, dans la section « Ouverture au réseau
# local » : il dépend des adresses IPv4 détectées sur la machine.

INSTALLED_APPS = [
    'django.contrib.admin',
    'django.contrib.auth',
    'django.contrib.contenttypes',
    'django.contrib.sessions',
    'django.contrib.messages',
    'django.contrib.staticfiles',
    'rest_framework',
    'corsheaders',
    'tenants',
    'licenses',
    'devices',
    'dashboard',
]

MIDDLEWARE = [
    'django.middleware.security.SecurityMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'corsheaders.middleware.CorsMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
    'django.contrib.messages.middleware.MessageMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
]

ROOT_URLCONF = 'config.urls'

TEMPLATES = [
    {
        'BACKEND': 'django.template.backends.django.DjangoTemplates',
        'DIRS': [BASE_DIR / 'templates'],
        'APP_DIRS': True,
        'OPTIONS': {
            'context_processors': [
                'django.template.context_processors.request',
                'django.contrib.auth.context_processors.auth',
                'django.contrib.messages.context_processors.messages',
            ],
        },
    },
]

WSGI_APPLICATION = 'config.wsgi.application'

DATABASES = {
    'default': {
        'ENGINE': 'django.db.backends.postgresql',
        'NAME': config('DB_NAME'),
        'USER': config('DB_USER'),
        'PASSWORD': config('DB_PASSWORD'),
        'HOST': config('DB_HOST', default='localhost'),
        'PORT': config('DB_PORT', default='5432'),
        'OPTIONS': {
            'options': '-c synchronous_commit=local',
        },
    }
}

AUTH_PASSWORD_VALIDATORS = [
    {'NAME': 'django.contrib.auth.password_validation.UserAttributeSimilarityValidator'},
    {'NAME': 'django.contrib.auth.password_validation.MinimumLengthValidator'},
    {'NAME': 'django.contrib.auth.password_validation.CommonPasswordValidator'},
    {'NAME': 'django.contrib.auth.password_validation.NumericPasswordValidator'},
]

LANGUAGE_CODE = 'en-us'
TIME_ZONE = 'UTC'
USE_I18N = True
USE_TZ = True

STATIC_URL = '/static/'
STATIC_ROOT = BASE_DIR / 'staticfiles'
MEDIA_URL = '/media/'
MEDIA_ROOT = BASE_DIR / 'media'

DEFAULT_AUTO_FIELD = 'django.db.models.BigAutoField'

# Django REST Framework — API consommée par la console React (session + CSRF,
# jamais de JWT). `IsAuthenticated` par défaut : les vues qui doivent rester
# publiques (register/, csrf/, login/, me/, devices/register/,
# devices/cluster-status/) déclarent explicitement `permission_classes = [AllowAny]`.
REST_FRAMEWORK = {
    'DEFAULT_AUTHENTICATION_CLASSES': [
        'rest_framework.authentication.SessionAuthentication',
    ],
    'DEFAULT_PERMISSION_CLASSES': [
        'rest_framework.permissions.IsAuthenticated',
    ],
}

# ---------------------------------------------------------------------------
# Ouverture au réseau local (banc de test)
# ---------------------------------------------------------------------------
# Le PC de l'éditeur sert de serveur : les autres machines du réseau ouvrent la
# console en tapant son IP. Cette IP n'est pas connue à l'avance et change selon
# l'interface (Wi-Fi, Ethernet, VMnet…), donc on ne peut pas la coder en dur.
# On énumère les adresses IPv4 locales au démarrage et on en dérive les origines
# de confiance. `LAN_PORTS` liste les ports par lesquels on peut entrer.

LAN_PORTS = ['5173', '8000', '80', '3000']


def _adresses_ipv4_locales() -> list[str]:
    """Adresses IPv4 de cette machine, hors loopback et hors APIPA.

    Best-effort : une interface peut disparaître (Wi-Fi coupé), et l'appel ne
    doit jamais empêcher Django de démarrer — d'où le filet large.
    """
    import socket

    adresses: set[str] = set()

    # 1. Adresses associées au nom d'hôte. Rapide, mais incomplet : Windows n'y
    #    fait pas toujours figurer toutes les cartes (l'Ethernet manque souvent).
    try:
        for info in socket.getaddrinfo(socket.gethostname(), None, socket.AF_INET):
            ip = info[4][0]
            if not ip.startswith(('127.', '169.254.')):
                adresses.add(ip)
    except OSError:
        pass

    # 2. Adresse de la route par défaut : on « connecte » un socket UDP vers une
    #    adresse externe pour demander au système quelle carte il emprunterait.
    #    Aucun paquet n'est émis (UDP est sans connexion), donc pas besoin d'un
    #    réseau réellement joignable — c'est la carte qui porte l'IP que les
    #    autres machines du réseau utiliseront pour joindre ce serveur.
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.connect(('8.8.8.8', 80))
        ip = sock.getsockname()[0]
        if not ip.startswith(('127.', '169.254.')):
            adresses.add(ip)
    except OSError:
        pass
    finally:
        sock.close()

    return sorted(adresses)


ADRESSES_LAN = _adresses_ipv4_locales()

# ALLOWED_HOSTS : les hôtes locaux, les noms de service compose, et toute IP
# locale détectée. Surchargeable par .env ; « * » y est accepté pour un banc de
# test où les IP changent souvent (à ne jamais utiliser en production).
ALLOWED_HOSTS = [
    h.strip() for h in config(
        'ALLOWED_HOSTS',
        default='localhost,127.0.0.1,backend,saas_editor_backend,host.docker.internal',
    ).split(',') if h.strip()
] + ADRESSES_LAN

# Origines autorisées à appeler l'API en cookies de session. `CORS_ALLOW_ALL_ORIGINS`
# est incompatible avec `CORS_ALLOW_CREDENTIALS` (cookies de session + CSRF) : il
# faut une liste explicite. Configurable via .env, séparée par des virgules.
#
# Depuis la mise en place du proxy nginx (frontend/nginx.conf), la console et
# l'API partagent la MÊME origine : le navigateur ne fait plus de requête
# cross-origin, et CORS ne joue plus aucun rôle dans le chemin nominal. Ces
# listes ne servent plus qu'aux accès directs au backend (:8000) et au serveur
# de développement Vite (:5173 en `npm run dev`).
FRONTEND_ORIGINS = [
    o.strip() for o in config(
        'FRONTEND_ORIGINS',
        default='http://localhost:5173,http://127.0.0.1:5173,http://localhost,http://127.0.0.1',
    ).split(',') if o.strip()
]

# Une entrée par IP locale et par port d'entrée : Django exige des origines
# exactes dans CSRF_TRUSTED_ORIGINS (le joker ne fonctionne que sur des
# sous-domaines, jamais sur des adresses IP).
ORIGINES_LAN = [
    f'http://{ip}:{port}' for ip in ADRESSES_LAN for port in LAN_PORTS
]

CORS_ALLOWED_ORIGINS = FRONTEND_ORIGINS + ORIGINES_LAN
CORS_ALLOW_CREDENTIALS = True
CSRF_TRUSTED_ORIGINS = FRONTEND_ORIGINS + ORIGINES_LAN

# Cookies de session/CSRF en clair : le banc de test est en HTTP sur le réseau
# local. Repasser ces deux réglages à True dès qu'un HTTPS est en place.
SESSION_COOKIE_SECURE = config('SESSION_COOKIE_SECURE', default=False, cast=bool)
CSRF_COOKIE_SECURE = config('CSRF_COOKIE_SECURE', default=False, cast=bool)

# nginx termine la connexion et relaie vers Django : sans cela, `request.is_secure()`
# et les URL construites côté Django ignoreraient le schéma vu par le navigateur.
USE_X_FORWARDED_HOST = True
SECURE_PROXY_SSL_HEADER = ('HTTP_X_FORWARDED_PROTO', 'https')

# Relais éditeur (ss-relay) — URL accessible depuis les conteneurs PME
RELAY_URL = config('RELAY_URL', default='http://host.docker.internal:8080')

# Registre Docker de l'éditeur (service compose `registry`). Les machines PME y
# tirent l'image du nœud. Seul le PORT est configuré : l'hôte est toujours
# dérivé de la requête entrante, car il dépend de l'IP par laquelle la PME a
# joint l'éditeur.
REGISTRY_PORT = config('REGISTRY_PORT', default='5000')

# URL par laquelle le BACKEND joint le registre pour l'interroger. À ne pas
# confondre avec l'hôte public ci-dessus : Django tourne dans un conteneur, d'où
# l'IP LAN de la machine hôte n'est pas routable. Sonder « 10.x.x.x:5000 » depuis
# le conteneur échoue donc toujours, et la console afficherait « registre
# injoignable » alors qu'il fonctionne. On passe par le nom de service compose.
REGISTRY_INTERNAL_URL = config('REGISTRY_INTERNAL_URL', default='http://registry:5000')

# Image Docker du nœud PME. `{registry}` est remplacé à la volée par
# « <hôte de la requête>:<REGISTRY_PORT> » au moment de générer le
# docker-compose.yml et les installeurs d'une PME.
SS_NODE_IMAGE = config('SS_NODE_IMAGE', default='{registry}/ss-node:v2')
