"""Frontière des rôles de l'API REST — voir CONTRAT-API.md.

Ces tests verrouillent le point le plus sensible de la migration « 100 % React » :
n'importe qui peut créer un compte PME via `POST /api/auth/register/`, qui est
public. Si les vues de l'espace éditeur ne portaient que le `IsAuthenticated`
par défaut de REST_FRAMEWORK, ce compte PME lirait et supprimerait les tenants,
licences et appareils de toutes les autres entreprises.

Note sur les codes : avec `SessionAuthentication` seule, DRF répond **403** à un
anonyme (et non 401) — `SessionAuthentication` ne fournit pas d'en-tête
`WWW-Authenticate`. Seule `/api/auth/me/` renvoie un 401 explicite, car elle le
construit elle-même pour que la console distingue « session absente » de
« session présente mais rôle insuffisant ».
"""
import json
import tempfile
from datetime import date, timedelta
from pathlib import Path

from django.contrib.auth.models import User
from django.test import TestCase, override_settings
from rest_framework import status

from devices.models import Device
from licenses.models import License
from tenants.models import Tenant

# Routes de l'espace éditeur : aucune ne doit répondre à un compte PME.
EDITOR_ENDPOINTS = [
    '/api/tenants/',
    '/api/licenses/',
    '/api/devices/',
    '/api/dashboard/stats/',
    '/api/clusters/',
]


class RoleBoundaryTests(TestCase):
    """Chaque route doit refuser l'anonyme et le mauvais rôle."""

    @classmethod
    def setUpTestData(cls):
        cls.staff = User.objects.create_user(
            username='editeur', password='motdepasse-editeur', is_staff=True,
        )

        cls.pme_user = User.objects.create_user(
            username='pme', password='motdepasse-pme',
        )
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='contact@boulangerie-martin.fr',
            employee_count=5,
            user=cls.pme_user,
        )
        cls.license = License.objects.create(
            tenant=cls.tenant,
            plan='starter',
            seats=5,
            starts_at=date.today(),
            expires_at=date.today() + timedelta(days=30),
        )
        cls.device = Device.objects.create(
            tenant=cls.tenant,
            installation_id='11111111-1111-1111-1111-111111111111',
            hostname='poste-caisse',
        )

        # Un second tenant, sans compte utilisateur : c'est celui-là qu'un
        # compte PME ne doit surtout pas pouvoir lire ni supprimer.
        cls.autre_tenant = Tenant.objects.create(
            name='Garage Dupont',
            email='contact@garage-dupont.fr',
            employee_count=3,
        )

    # -- anonyme ------------------------------------------------------------

    def test_espace_editeur_refuse_anonyme(self):
        for url in EDITOR_ENDPOINTS:
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_403_FORBIDDEN,
                )

    def test_espace_pme_refuse_anonyme(self):
        self.assertEqual(
            self.client.get('/api/tenants/me/').status_code,
            status.HTTP_403_FORBIDDEN,
        )

    # -- compte PME sur l'espace éditeur ------------------------------------

    def test_compte_pme_refuse_sur_espace_editeur(self):
        self.client.force_login(self.pme_user)
        for url in EDITOR_ENDPOINTS:
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_403_FORBIDDEN,
                )

    def test_compte_pme_ne_peut_pas_supprimer_un_autre_tenant(self):
        self.client.force_login(self.pme_user)
        response = self.client.delete(f'/api/tenants/{self.autre_tenant.id}/')
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)
        self.assertTrue(Tenant.objects.filter(pk=self.autre_tenant.pk).exists())

    def test_compte_pme_ne_peut_pas_creer_de_licence(self):
        self.client.force_login(self.pme_user)
        response = self.client.post('/api/licenses/', {
            'tenant': str(self.tenant.id),
            'plan': 'enterprise',
            'seats': 5,
            'starts_at': date.today().isoformat(),
        })
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)
        self.assertEqual(self.tenant.licenses.count(), 1)

    def test_compte_pme_ne_peut_pas_basculer_un_appareil(self):
        self.client.force_login(self.pme_user)
        response = self.client.post(f'/api/devices/{self.device.id}/toggle/')
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)
        self.device.refresh_from_db()
        self.assertTrue(self.device.is_active)

    # -- compte éditeur sur l'espace PME ------------------------------------

    def test_compte_editeur_refuse_sur_espace_pme(self):
        self.client.force_login(self.staff)
        for url in ('/api/tenants/me/', '/api/tenants/me/onboarding/'):
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_403_FORBIDDEN,
                )

    # -- chemins autorisés ---------------------------------------------------

    def test_compte_editeur_accede_a_lespace_editeur(self):
        self.client.force_login(self.staff)
        for url in EDITOR_ENDPOINTS:
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_200_OK,
                )

    def test_compte_pme_accede_a_son_espace(self):
        self.client.force_login(self.pme_user)
        response = self.client.get('/api/tenants/me/')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.json()['tenant']['name'], 'Boulangerie Martin')

    def test_editeur_voit_bien_tous_les_tenants(self):
        self.client.force_login(self.staff)
        noms = {t['name'] for t in self.client.get('/api/tenants/').json()}
        self.assertEqual(noms, {'Boulangerie Martin', 'Garage Dupont'})


class AuthEndpointTests(TestCase):
    """`/api/auth/` — forme de l'objet User et cycle login/me/logout."""

    @classmethod
    def setUpTestData(cls):
        cls.staff = User.objects.create_user(
            username='editeur', password='motdepasse-editeur', is_staff=True,
        )
        cls.pme_user = User.objects.create_user(
            username='pme', password='motdepasse-pme',
        )
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='contact@boulangerie-martin.fr',
            employee_count=5,
            user=cls.pme_user,
        )

    def test_me_sans_session_renvoie_401(self):
        """401 et non 403 : la console distingue « pas de session » du reste."""
        self.assertEqual(
            self.client.get('/api/auth/me/').status_code,
            status.HTTP_401_UNAUTHORIZED,
        )

    def test_login_renvoie_le_role_editor(self):
        response = self.client.post('/api/auth/login/', {
            'username': 'editeur', 'password': 'motdepasse-editeur',
        })
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['role'], 'editor')
        self.assertTrue(data['is_staff'])
        self.assertIsNone(data['tenant'])

    def test_login_renvoie_le_role_tenant_et_son_entreprise(self):
        response = self.client.post('/api/auth/login/', {
            'username': 'pme', 'password': 'motdepasse-pme',
        })
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertEqual(data['role'], 'tenant')
        self.assertFalse(data['is_staff'])
        self.assertEqual(data['tenant']['name'], 'Boulangerie Martin')
        self.assertEqual(
            data['tenant']['registration_token'], str(self.tenant.registration_token),
        )

    def test_login_invalide_renvoie_400_avec_detail(self):
        response = self.client.post('/api/auth/login/', {
            'username': 'editeur', 'password': 'mauvais',
        })
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('detail', response.json())

    def test_logout_ferme_la_session(self):
        self.client.force_login(self.staff)
        self.assertEqual(
            self.client.post('/api/auth/logout/').status_code,
            status.HTTP_204_NO_CONTENT,
        )
        self.assertEqual(
            self.client.get('/api/auth/me/').status_code,
            status.HTTP_401_UNAUTHORIZED,
        )

    def test_csrf_pose_le_cookie(self):
        response = self.client.get('/api/auth/csrf/')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn('csrftoken', response.cookies)


class RegisterTests(TestCase):
    """`POST /api/auth/register/` — même logique que l'inscription web."""

    PAYLOAD = {
        'name': 'Garage Dupont',
        'email': 'contact@garage-dupont.fr',
        'phone': '0102030405',
        'address': '1 rue des Ateliers',
        'employee_count': 4,
        'password': 'MotDePasseSolide!2026',
        'password_confirm': 'MotDePasseSolide!2026',
    }

    def test_inscription_cree_compte_tenant_et_licence_dessai(self):
        response = self.client.post('/api/auth/register/', self.PAYLOAD)
        self.assertEqual(response.status_code, status.HTTP_201_CREATED)

        data = response.json()
        self.assertEqual(data['role'], 'tenant')
        self.assertEqual(data['tenant']['name'], 'Garage Dupont')

        tenant = Tenant.objects.get(email='contact@garage-dupont.fr')
        self.assertEqual(tenant.employee_count, 4)

        licence = tenant.licenses.get()
        self.assertEqual(licence.plan, 'starter')
        self.assertEqual(licence.seats, 4)
        self.assertTrue(licence.is_active)

    def test_inscription_connecte_directement(self):
        self.client.post('/api/auth/register/', self.PAYLOAD)
        response = self.client.get('/api/auth/me/')
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.json()['tenant']['name'], 'Garage Dupont')

    def test_inscription_refuse_un_email_deja_pris(self):
        self.client.post('/api/auth/register/', self.PAYLOAD)

        response = self.client.post('/api/auth/register/', self.PAYLOAD)
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        # Erreurs champ par champ : c'est la forme qu'attend normalizeApiError().
        self.assertIn('email', response.json())
        self.assertEqual(Tenant.objects.filter(email=self.PAYLOAD['email']).count(), 1)

    def test_inscription_refuse_des_mots_de_passe_differents(self):
        payload = {**self.PAYLOAD, 'password_confirm': 'autre-chose'}
        response = self.client.post('/api/auth/register/', payload)
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertFalse(Tenant.objects.filter(email=payload['email']).exists())


class PublicNodeEndpointTests(TestCase):
    """Endpoints appelés par les nœuds PME eux-mêmes : publics par contrat."""

    @classmethod
    def setUpTestData(cls):
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='contact@boulangerie-martin.fr',
            employee_count=5,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=2, starts_at=date.today(),
        )

    def test_register_autorise_un_noeud_sans_session(self):
        response = self.client.post('/api/devices/register/', {
            'tenant_token': str(self.tenant.registration_token),
            'installation_id': '22222222-2222-2222-2222-222222222222',
            'hostname': 'noeud-1',
        })
        self.assertEqual(response.status_code, status.HTTP_201_CREATED)
        self.assertEqual(response.json()['status'], 'authorized')

    def test_register_refuse_un_token_inconnu(self):
        response = self.client.post('/api/devices/register/', {
            'tenant_token': '33333333-3333-3333-3333-333333333333',
            'installation_id': '44444444-4444-4444-4444-444444444444',
        })
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)
        self.assertEqual(response.json()['reason'], 'invalid_token')

    def test_cluster_status_lisible_sans_session(self):
        response = self.client.get(
            '/api/devices/cluster-status/',
            {'token': str(self.tenant.registration_token)},
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertEqual(response.json()['tenant_name'], 'Boulangerie Martin')


class NodeImageEndpointTests(TestCase):
    """`GET /api/node-image/` et `GET /api/node-image/download/` — voir
    CONTRAT-API.md section 2. Manifeste ou archive absents sont des états
    normaux (200, `disponible: false`), jamais une 500 — même logique que le
    relais injoignable dans `dashboard/api_relay.py`.
    """

    @classmethod
    def setUpTestData(cls):
        cls.staff = User.objects.create_user(
            username='editeur-images', password='motdepasse-editeur', is_staff=True,
        )
        cls.pme_user = User.objects.create_user(
            username='pme-images', password='motdepasse-pme',
        )
        Tenant.objects.create(
            name='Boulangerie Martin',
            email='images@boulangerie-martin.fr',
            employee_count=5,
            user=cls.pme_user,
        )

    def setUp(self):
        tmp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(tmp_dir.cleanup)
        self.media_root = Path(tmp_dir.name)

    def _write_manifest(self, **overrides):
        images_dir = self.media_root / 'node-images'
        images_dir.mkdir(parents=True, exist_ok=True)
        manifest = {
            'version': 'v2',
            'image': 'ss-node:v2',
            'tag_registre': 'ss-node:v2',
            'fichier': 'ss-node-v2.tar.gz',
            'taille_octets': 123456789,
            'sha256': 'a' * 64,
            'construit_le': '2026-08-31T18:30:00Z',
        }
        manifest.update(overrides)
        (images_dir / 'manifeste.json').write_text(json.dumps(manifest), encoding='utf-8')
        return manifest

    def _write_archive(self, filename='ss-node-v2.tar.gz'):
        images_dir = self.media_root / 'node-images'
        images_dir.mkdir(parents=True, exist_ok=True)
        (images_dir / filename).write_bytes(b'contenu-factice-de-l-archive')

    # -- rôles ----------------------------------------------------------

    def test_anonyme_refuse_sur_les_deux_routes(self):
        for url in ('/api/node-image/', '/api/node-image/download/'):
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_403_FORBIDDEN,
                )

    def test_compte_pme_refuse_sur_les_deux_routes(self):
        self.client.force_login(self.pme_user)
        for url in ('/api/node-image/', '/api/node-image/download/'):
            with self.subTest(url=url):
                self.assertEqual(
                    self.client.get(url).status_code, status.HTTP_403_FORBIDDEN,
                )

    # -- manifeste absent -------------------------------------------------

    def test_editeur_manifeste_absent_renvoie_disponible_false(self):
        self.client.force_login(self.staff)
        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get('/api/node-image/')

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertFalse(data['disponible'])
        self.assertIsNone(data['version'])
        self.assertIsNone(data['image'])
        self.assertIsNone(data['sha256'])
        self.assertIsNone(data['construit_le'])
        self.assertIn('registre', data)
        self.assertFalse(data['registre']['joignable'])

    # -- manifeste et archive présents -------------------------------------

    def test_editeur_manifeste_et_archive_presents_renvoie_les_bonnes_valeurs(self):
        self._write_manifest()
        self._write_archive()
        self.client.force_login(self.staff)

        with override_settings(
            MEDIA_ROOT=str(self.media_root),
            ALLOWED_HOSTS=['testserver', '192.168.7.7'],
        ):
            response = self.client.get('/api/node-image/', HTTP_HOST='192.168.7.7:8000')

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        data = response.json()
        self.assertTrue(data['disponible'])
        self.assertEqual(data['version'], 'v2')
        self.assertEqual(data['image'], 'ss-node:v2')
        self.assertEqual(data['fichier'], 'ss-node-v2.tar.gz')
        self.assertEqual(data['taille_octets'], 123456789)
        self.assertEqual(data['sha256'], 'a' * 64)
        self.assertEqual(data['construit_le'], '2026-08-31T18:30:00Z')
        self.assertEqual(data['url_telechargement'], '/api/node-image/download/')
        self.assertIn('192.168.7.7', data['registre']['hote'])
        self.assertIn('docker pull', data['registre']['commande_pull'])
        self.assertIn('192.168.7.7', data['registre']['commande_pull'])
        self.assertIn('ss-node:v2', data['registre']['commande_pull'])

    # -- téléchargement -----------------------------------------------------

    def test_telechargement_archive_absente_renvoie_404(self):
        self._write_manifest()
        self.client.force_login(self.staff)

        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get('/api/node-image/download/')

        self.assertEqual(response.status_code, status.HTTP_404_NOT_FOUND)
        self.assertIn('detail', response.json())

    def test_telechargement_manifeste_absent_renvoie_404(self):
        self.client.force_login(self.staff)

        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get('/api/node-image/download/')

        self.assertEqual(response.status_code, status.HTTP_404_NOT_FOUND)

    def test_telechargement_reussi_sert_larchive_en_streaming(self):
        self._write_manifest()
        self._write_archive()
        self.client.force_login(self.staff)

        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get('/api/node-image/download/')

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn('attachment', response['Content-Disposition'])
        self.assertIn('ss-node-v2.tar.gz', response['Content-Disposition'])
        self.assertEqual(b''.join(response.streaming_content), b'contenu-factice-de-l-archive')


class NodeImagePmeDownloadTests(TestCase):
    """`GET /api/node-image/telecharger/?token=…` — voie des installeurs PME.

    Route publique authentifiée par le jeton d'enregistrement du tenant, comme
    `devices/api_cluster_status.py` : la machine qui déploie le nœud n'a aucune
    session éditeur. C'est aussi le repli quand le registre Docker est fermé ou
    qu'un poste Windows ne peut pas le déclarer en `insecure-registries`.
    """

    URL = '/api/node-image/telecharger/'

    @classmethod
    def setUpTestData(cls):
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='pme-image@boulangerie-martin.fr',
            employee_count=5,
        )
        cls.tenant_inactif = Tenant.objects.create(
            name='Garage Ferme',
            email='ferme@garage.fr',
            employee_count=2,
            is_active=False,
        )

    def setUp(self):
        tmp_dir = tempfile.TemporaryDirectory()
        self.addCleanup(tmp_dir.cleanup)
        self.media_root = Path(tmp_dir.name)
        self.images_dir = self.media_root / 'node-images'
        self.images_dir.mkdir(parents=True, exist_ok=True)

    def _publier_image(self):
        (self.images_dir / 'manifeste.json').write_text(json.dumps({
            'version': 'v2',
            'image': 'ss-node:v2',
            'tag_registre': 'ss-node:v2',
            'fichier': 'ss-node-v2.tar.gz',
            'taille_octets': 28,
            'sha256': 'b' * 64,
            'construit_le': '2026-08-31T20:03:02Z',
        }), encoding='utf-8')
        (self.images_dir / 'ss-node-v2.tar.gz').write_bytes(b'contenu-factice-de-l-archive')

    def test_sans_token_renvoie_400(self):
        response = self.client.get(self.URL)
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)

    def test_token_mal_forme_renvoie_400(self):
        response = self.client.get(self.URL, {'token': 'pas-un-uuid'})
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)

    def test_token_inconnu_renvoie_403(self):
        response = self.client.get(
            self.URL, {'token': '33333333-3333-3333-3333-333333333333'},
        )
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)

    def test_tenant_inactif_renvoie_403(self):
        response = self.client.get(
            self.URL, {'token': str(self.tenant_inactif.registration_token)},
        )
        self.assertEqual(response.status_code, status.HTTP_403_FORBIDDEN)

    def test_token_valide_sans_image_publiee_renvoie_404(self):
        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get(
                self.URL, {'token': str(self.tenant.registration_token)},
            )
        self.assertEqual(response.status_code, status.HTTP_404_NOT_FOUND)

    def test_token_valide_sert_larchive_sans_session(self):
        """Le cas nominal : aucune session, seulement le jeton du tenant."""
        self._publier_image()

        with override_settings(MEDIA_ROOT=str(self.media_root)):
            response = self.client.get(
                self.URL, {'token': str(self.tenant.registration_token)},
            )

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn('attachment', response['Content-Disposition'])
        self.assertIn('ss-node-v2.tar.gz', response['Content-Disposition'])
        self.assertEqual(
            b''.join(response.streaming_content), b'contenu-factice-de-l-archive',
        )


class FichiersGeneresPourLaPmeTests(TestCase):
    """Le docker-compose.yml et les installeurs servis à une PME doivent
    pointer le registre de l'ÉDITEUR, à l'adresse par laquelle cette PME vient
    de le joindre — jamais « localhost », qui désignerait la machine de la PME.

    C'est le point qui casse silencieusement le déploiement réseau : un fichier
    généré avec une adresse figée fonctionne sur le poste de l'éditeur et échoue
    partout ailleurs.
    """

    @classmethod
    def setUpTestData(cls):
        cls.pme_user = User.objects.create_user(
            username='pme-fichiers', password='motdepasse-pme',
        )
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='fichiers@boulangerie-martin.fr',
            employee_count=5,
            user=cls.pme_user,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )

    def setUp(self):
        self.client.force_login(self.pme_user)

    def _corps(self, response) -> str:
        if response.streaming:
            return b''.join(response.streaming_content).decode('utf-8')
        return response.content.decode('utf-8')

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_compose_pointe_le_registre_de_lediteur(self):
        response = self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)

        corps = self._corps(response)
        self.assertIn('image: 192.168.7.7:5000/ss-node:v2', corps)
        # Le marqueur doit avoir été résolu, pas recopié tel quel.
        self.assertNotIn('{registry}', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_installeur_linux_declare_le_registre_non_securise(self):
        response = self.client.get(
            '/api/tenants/me/download/installer/linux/', HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)

        corps = self._corps(response)
        self.assertIn('192.168.7.7:5000/ss-node:v2', corps)
        # Sans insecure-registries, Docker exigerait du TLS et le pull échouerait.
        self.assertIn('REGISTRY_HOST="192.168.7.7:5000"', corps)
        self.assertIn('insecure-registries', corps)
        self.assertNotIn('{registry}', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_installeur_windows_gere_registre_et_repli_archive(self):
        response = self.client.get(
            '/api/tenants/me/download/installer/windows/', HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)

        corps = self._corps(response)
        self.assertIn('192.168.7.7:5000/ss-node:v2', corps)
        self.assertIn('insecure-registries', corps)
        # Repli quand le registre est injoignable : archive + docker load.
        self.assertIn('/api/node-image/telecharger/', corps)
        self.assertIn('docker load', corps)
        self.assertNotIn('{registry}', corps)


    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_compose_premier_noeud_est_actif(self):
        """Sans primaire existant, le compose part en « active ».

        C'est `NODE_MODE` — et non l'argument `--mode` — que lit le noeud pour
        decider du role qu'il annonce, et l'entrypoint de l'image pour choisir
        `ss-node init --first`, qui genere la DEK. Quand la variable manquait,
        tous les noeuds s'annoncaient `standby` et aucun ne generait de cle.
        """
        self.assertFalse(
            self.tenant.devices.filter(is_active=True, node_role='primary').exists()
        )
        corps = self._corps(self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        ))
        self.assertIn('NODE_MODE: "${NODE_MODE:-active}"', corps)
        self.assertIn('"--mode", "${NODE_MODE:-active}"', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_compose_noeud_suivant_est_passif(self):
        """Un primaire deja en place bascule le fichier suivant en « passive ».

        Evite le split-brain : deux noeuds partis en « active » sur le meme
        cluster se declareraient tous deux primaires.
        """
        Device.objects.create(
            tenant=self.tenant,
            installation_id='99999999-9999-9999-9999-999999999999',
            hostname='node1',
            node_role='primary',
            is_active=True,
        )
        corps = self._corps(self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        ))
        self.assertIn('NODE_MODE: "${NODE_MODE:-passive}"', corps)
        self.assertNotIn('${NODE_MODE:-active}', corps)

    @override_settings(
        ALLOWED_HOSTS=['10.0.0.42'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_ladresse_suit_lhote_de_la_requete(self):
        """Une PME qui joint l'éditeur par une autre IP reçoit cette IP-là."""
        response = self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='10.0.0.42',
        )
        corps = self._corps(response)
        self.assertIn('image: 10.0.0.42:5000/ss-node:v2', corps)
        self.assertNotIn('192.168.7.7', corps)


class DetectionAdresseNoeudTests(TestCase):
    """Le SaaS complète l'adresse annoncée par le nœud avec l'IP source.

    Le nœud tourne dans un conteneur : il ne voit que son IP interne (172.x) et
    ne peut pas découvrir l'IP LAN de sa machine hôte — celle par laquelle les
    autres nœuds le joindront. Le serveur, lui, la lit dans la connexion
    entrante. Sans cela, il fallait éditer `NODE_ADDR` à la main dans le
    docker-compose.yml, et un fichier laissé tel quel enregistrait le nœud avec
    une adresse littéralement invalide.
    """

    URL = '/api/devices/register/'

    @classmethod
    def setUpTestData(cls):
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='adresse@boulangerie-martin.fr',
            employee_count=5,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )

    def _enregistrer(self, installation_id, **extra):
        corps = {
            'tenant_token': str(self.tenant.registration_token),
            'installation_id': installation_id,
        }
        corps.update(extra)
        return self.client.post(self.URL, corps, REMOTE_ADDR='10.128.177.77')

    def test_adresse_du_reseau_docker_est_ecartee(self):
        """Docker Desktop (Windows/macOS) reecrit l'IP source a la passerelle du
        reseau bridge : toutes les machines arrivent avec la meme adresse. On
        prefere un champ vide, qui se voit, a une adresse fausse mais credible.
        """
        r = self.client.post(self.URL, {
            'tenant_token': str(self.tenant.registration_token),
            'installation_id': 'aaaaaaaa-0000-0000-0000-000000000006',
            'node_addr': '<IP_VM>:9001',
        }, REMOTE_ADDR='172.18.0.1')
        self.assertEqual(r.status_code, status.HTTP_201_CREATED)

        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000006')
        self.assertNotIn('172.18.0.1', d.node_addr)
        self.assertNotIn('<IP_VM>', d.node_addr)

    def test_adresse_vide_est_completee_par_lip_source(self):
        r = self._enregistrer('aaaaaaaa-0000-0000-0000-000000000001', node_addr='')
        self.assertEqual(r.status_code, status.HTTP_201_CREATED)

        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000001')
        self.assertEqual(d.node_addr, '10.128.177.77:9001')
        self.assertEqual(d.web_addr, '10.128.177.77:3000')

    def test_gabarit_non_remplace_est_remplace_par_lip_source(self):
        """Le cas qui piégeait : `<IP_VM>:9001` laissé tel quel."""
        r = self._enregistrer(
            'aaaaaaaa-0000-0000-0000-000000000002',
            node_addr='<IP_VM>:9001', web_addr='<IP_VM>:3000',
        )
        self.assertEqual(r.status_code, status.HTTP_201_CREATED)

        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000002')
        self.assertEqual(d.node_addr, '10.128.177.77:9001')
        self.assertNotIn('<IP_VM>', d.node_addr)
        self.assertNotIn('<IP_VM>', d.web_addr)

    def test_ip_explicite_du_noeud_est_respectee(self):
        """Une adresse réellement annoncée prime : machine multi-domiciliée."""
        r = self._enregistrer(
            'aaaaaaaa-0000-0000-0000-000000000003', node_addr='192.168.1.42:9001',
        )
        self.assertEqual(r.status_code, status.HTTP_201_CREATED)

        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000003')
        self.assertEqual(d.node_addr, '192.168.1.42:9001')

    def test_port_annonce_est_conserve(self):
        r = self._enregistrer(
            'aaaaaaaa-0000-0000-0000-000000000004', node_addr='<IP_VM>:9999',
        )
        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000004')
        self.assertEqual(d.node_addr, '10.128.177.77:9999')

    def test_ip_reelle_prise_derriere_le_proxy_nginx(self):
        """Derrière nginx, REMOTE_ADDR est le proxy : la vraie IP est en tête
        de X-Forwarded-For."""
        corps = {
            'tenant_token': str(self.tenant.registration_token),
            'installation_id': 'aaaaaaaa-0000-0000-0000-000000000005',
            'node_addr': '',
        }
        r = self.client.post(
            self.URL, corps,
            REMOTE_ADDR='172.18.0.5',
            HTTP_X_FORWARDED_FOR='10.128.177.88, 172.18.0.5',
        )
        self.assertEqual(r.status_code, status.HTTP_201_CREATED)

        d = Device.objects.get(installation_id='aaaaaaaa-0000-0000-0000-000000000005')
        self.assertEqual(d.node_addr, '10.128.177.88:9001')


class InstalleurWindowsTests(TestCase):
    """L'installeur Windows doit rester en ASCII PUR.

    `cmd` lit un fichier .bat par decalage d'octets. Un caractere multi-octets
    (accent, guillemet francais) lui fait perdre sa position apres un `goto` :
    il reprend la lecture au milieu d'une ligne, execute des fragments
    tronques, et la fenetre se ferme sans message d'erreur exploitable.
    Symptome constate sur le banc : le script affichait sa banniere puis
    disparaissait. Le reste du gabarit etait deja en ASCII pour cette raison.
    """

    @classmethod
    def setUpTestData(cls):
        cls.pme_user = User.objects.create_user(
            username='pme-win', password='motdepasse-pme',
        )
        # Nom volontairement accentue : c'est la seule donnee variable qui
        # pourrait reintroduire du non-ASCII dans le .bat.
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Éclair & Cie — Crémerie',
            email='windows@boulangerie-martin.fr',
            employee_count=5,
            user=cls.pme_user,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )

    def setUp(self):
        self.client.force_login(self.pme_user)

    def _installeur(self) -> str:
        response = self.client.get(
            '/api/tenants/me/download/installer/windows/', HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(response.status_code, status.HTTP_200_OK)
        if response.streaming:
            return b''.join(response.streaming_content).decode('utf-8')
        return response.content.decode('utf-8')

    def test_installeur_est_en_ascii_pur(self):
        contenu = self._installeur()
        hors_ascii = sorted({c for c in contenu if ord(c) > 127})
        self.assertEqual(
            hors_ascii, [],
            f"Caracteres non-ASCII dans l'installeur .bat : {hors_ascii}",
        )

    def test_nom_du_tenant_est_replie_en_ascii(self):
        """Le nom accentue doit apparaitre, deponctue de ses accents."""
        contenu = self._installeur()
        self.assertIn('Boulangerie Eclair', contenu)
        self.assertNotIn('Éclair', contenu)

    def test_aucun_label_dans_un_bloc_parenthese(self):
        """Un label dans un bloc `if (...)` corrompt le flux : apres la boucle,
        cmd execute AUSSI la branche `else`."""
        profondeur = 0
        for numero, ligne in enumerate(self._installeur().splitlines(), 1):
            nue = ligne.strip()
            if nue.startswith(':') and not nue.startswith('::') and profondeur > 0:
                self.fail(f"Label '{nue}' dans un bloc parenthese, ligne {numero}")
            profondeur += ligne.count('(') - ligne.count(')')
            profondeur = max(profondeur, 0)

    def test_aucun_tube_echappe_dans_un_for_f(self):
        """Un « ^| » dans un for /f qui appelle powershell casse tout.

        cmd n'applique PAS l'echappement par caret a l'interieur des
        guillemets : le « ^ » arrivait tel quel a PowerShell, qui refusait
        d'analyser la commande. La variable restait vide, EN SILENCE. Effets
        constates : la detection d'IP retombait sur 127.0.0.1 et la
        declaration du registre non securise n'etait jamais ecrite, d'ou des
        « docker pull » qui echouaient en HTTPS. Le tube s'ecrit « | » nu.
        """
        contenu = self._installeur()
        fautives = [
            (numero, ligne)
            for numero, ligne in enumerate(contenu.splitlines(), 1)
            if '^|' in ligne and not ligne.strip().startswith('::')
        ]
        self.assertEqual(
            fautives, [],
            'Tube echappe hors commentaire : ' + repr(fautives),
        )

    def test_node_mode_ecrit_dans_le_env_et_declare_dans_le_compose(self):
        """Remplace `test_compose_genere_porte_node_mode`.

        L'installeur ne genere plus le docker-compose.yml : il le telecharge
        depuis le portail et ne fournit que le `.env`. Le controle reste le
        meme, deplace d'un cran : NODE_MODE doit etre ECRIT dans le .env par le
        .bat, et LU par le gabarit de compose. Sans lui, le noeud s'annonce
        toujours standby et n'execute jamais `ss-node init --first`, qui genere
        la DEK.
        """
        contenu = self._installeur()
        # 1. Le .bat calcule le mode et l'ecrit dans le .env.
        self.assertIn('set "NODE_MODE=active"', contenu)
        self.assertIn('set "NODE_MODE=passive"', contenu)
        self.assertIn('echo NODE_MODE=!NODE_MODE!', contenu)
        # 2. Il telecharge bien le compose qui consommera cette variable.
        self.assertIn('/api/tenants/pme/compose/', contenu)
        # 3. Et le gabarit de compose la lit, pour l'environnement du noeud
        #    comme pour l'argument --mode.
        compose = self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        ).content.decode('utf-8')
        self.assertIn('NODE_MODE: "${NODE_MODE:-', compose)
        self.assertIn('"--mode", "${NODE_MODE:-', compose)


class ComposeParJetonTests(TestCase):
    """`GET /api/tenants/pme/compose/?token=...` — meme contrat d'erreur que
    `dashboard.api_images.node_image_telecharger_pme`.

    C'est ce point d'entree qui fait du gabarit `docker-compose.yml.tmpl` la
    definition UNIQUE de la pile : les installeurs Windows et Linux le
    telechargent au lieu de le regenerer chacun a sa maniere. Ils tournent sur
    des machines sans compte editeur : l'authentification passe donc par le
    jeton d'enregistrement du tenant, jamais par une session.
    """

    URL = '/api/tenants/pme/compose/'

    @classmethod
    def setUpTestData(cls):
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='compose@boulangerie-martin.fr',
            employee_count=5,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )
        cls.tenant_inactif = Tenant.objects.create(
            name='Tenant suspendu',
            email='suspendu@exemple.fr',
            employee_count=2,
            is_active=False,
        )

    def _corps(self, response) -> str:
        if response.streaming:
            return b''.join(response.streaming_content).decode('utf-8')
        return response.content.decode('utf-8')

    def test_sans_token_renvoie_400(self):
        self.assertEqual(self.client.get(self.URL).status_code, 400)

    def test_token_mal_forme_renvoie_400(self):
        r = self.client.get(self.URL, {'token': 'pas-un-uuid'})
        self.assertEqual(r.status_code, 400)

    def test_token_inconnu_renvoie_403(self):
        r = self.client.get(
            self.URL, {'token': '00000000-0000-0000-0000-000000000000'},
        )
        self.assertEqual(r.status_code, 403)

    def test_tenant_inactif_renvoie_403(self):
        r = self.client.get(
            self.URL, {'token': str(self.tenant_inactif.registration_token)},
        )
        self.assertEqual(r.status_code, 403)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_token_valide_sert_la_pile_complete_sans_session(self):
        """Aucun `force_login` : l'installeur n'a pas de session editeur."""
        r = self.client.get(
            self.URL,
            {'token': str(self.tenant.registration_token)},
            HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(r.status_code, status.HTTP_200_OK)

        corps = self._corps(r)
        for service in ('postgres:', 'ss-node:', 'pgadmin:', 'updater:'):
            self.assertIn(service, corps, 'Service absent : ' + service)
        self.assertIn('image: postgres:16-bookworm', corps)
        self.assertIn('192.168.7.7:5000/ss-node:v2', corps)
        self.assertIn('containrrr/watchtower', corps)
        self.assertIn('PG_URL:', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_meme_contenu_que_la_voie_par_session(self):
        """Une seule definition de la pile : la voie par jeton et la voie par
        session doivent servir exactement le meme fichier."""
        user = User.objects.create_user(username='pme-compose', password='x')
        self.tenant.user = user
        self.tenant.save(update_fields=['user'])

        par_jeton = self._corps(self.client.get(
            self.URL,
            {'token': str(self.tenant.registration_token)},
            HTTP_HOST='192.168.7.7',
        ))
        self.client.force_login(user)
        par_session = self._corps(self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        ))

        # Seule la ligne d'horodatage peut differer d'un rendu a l'autre.
        def nettoyer(texte):
            return [l for l in texte.splitlines() if not l.startswith('# Genere')]

        self.assertEqual(nettoyer(par_jeton), nettoyer(par_session))


class PilePmeTests(TestCase):
    """Le gabarit de compose decrit une pile complete et exploitable.

    Ces controles verrouillent ce qui manquait cote Windows : sans postgres, le
    noeud n'a pas de PG_URL, donc pas de supervision, pas de bascule et aucune
    replication possible avec la machine Fedora du meme cluster.
    """

    @classmethod
    def setUpTestData(cls):
        cls.user = User.objects.create_user(username='pme-pile', password='x')
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='pile@boulangerie-martin.fr',
            employee_count=5,
            user=cls.user,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )

    def setUp(self):
        self.client.force_login(self.user)

    def test_le_nom_de_machine_nest_pas_fige(self):
        """Deux machines d'un meme parc doivent etre distinguables dans /devices.

        Le gabarit figeait « HOSTNAME: node1 » : les deux noeuds du cluster
        s'annoncaient sous le meme nom au portail, ce qui rendait la colonne
        Hostname inutilisable et empechait de savoir laquelle des deux machines
        etait primaire.
        """
        compose = self._compose()
        self.assertNotIn(
            'HOSTNAME: node1', compose,
            "le nom de machine ne doit plus etre code en dur dans la pile",
        )
        self.assertIn('${NODE_HOSTNAME:-node1}', compose)

    def _compose(self) -> str:
        r = self.client.get(
            '/api/tenants/me/download/compose/', HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        return r.content.decode('utf-8')

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_le_noeud_recoit_pg_url_et_attend_postgres(self):
        corps = self._compose()
        self.assertIn('PG_URL: "postgres://', corps)
        self.assertIn('@postgres:5432/', corps)
        self.assertIn('condition: service_healthy', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_options_de_replication_postgres(self):
        # max_wal_senders=5 : 2 standbys en flux + un pg_basebackup -Xs (qui
        # occupe 2 émetteurs) doivent pouvoir coexister sur un cluster de 3.
        # wal_keep_size=512 (Mo) : un standby redémarré rattrape son retard sans
        # être recloné. Mêmes valeurs que pme-deploy/docker-compose.yml.
        corps = self._compose()
        for option in (
            'wal_level=replica',
            'max_wal_senders=5',
            'wal_keep_size=512',
            'listen_addresses=*',
            'synchronous_commit=remote_write',
        ):
            self.assertIn(option, corps, 'Option absente : ' + option)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_initialisation_postgres_sans_fichier_annexe(self):
        """La logique primaire/standby est EN LIGNE dans le compose.

        Elle etait auparavant dans deux scripts bash ecrits par l'installeur
        Linux et montes en volume : irreproductible depuis un .bat sans se
        battre avec l'echappement de « > », « | » et « % ». Plus aucun
        `volumes:` ne doit pointer un fichier de l'hote.
        """
        corps = self._compose()
        self.assertIn('pg_basebackup', corps)
        self.assertIn('PG_VERSION', corps)
        self.assertIn('CREATE USER replicator WITH REPLICATION', corps)
        self.assertIn('host    replication     replicator', corps)
        self.assertIn('pg_ctl reload', corps)
        self.assertNotIn('- ./', corps)
        self.assertNotIn('entrypoint.sh:/entrypoint.sh', corps)

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def test_le_compose_est_un_yaml_valide(self):
        """`yaml.safe_load` doit accepter le rendu — le bloc `entrypoint` est
        un scalaire litteral multiligne, c'est la partie fragile."""
        try:
            import yaml
        except ImportError:  # pragma: no cover - PyYAML est optionnel ici
            self.skipTest('PyYAML absent de cet environnement')

        doc = yaml.safe_load(self._compose())
        self.assertEqual(
            sorted(doc['services']), ['pgadmin', 'postgres', 'ss-node', 'updater'],
        )
        self.assertEqual(
            sorted(doc['volumes']), ['node-data', 'pg-data', 'pgadmin-data'],
        )
        entrypoint = doc['services']['postgres']['entrypoint']
        self.assertEqual(entrypoint[0], '/bin/bash')
        self.assertEqual(entrypoint[1], '-c')
        self.assertEqual(entrypoint[-1], '--')


class PariteInstalleursTests(TestCase):
    """Windows et Linux doivent deployer une pile IDENTIQUE et interoperable.

    Le banc de test est un parc heterogene : machine 1 sous Windows, machine 2
    sous Fedora, meme tenant, meme cluster. Tant que l'installeur Windows ne
    posait pas de PostgreSQL, le noeud n'avait pas de PG_URL — donc ni
    supervision, ni bascule, ni replication avec la Fedora — et le parc etait
    intestable. Chaque assertion ci-dessous tombe si l'un des DEUX installeurs
    regresse.
    """

    @classmethod
    def setUpTestData(cls):
        cls.user = User.objects.create_user(username='pme-parite', password='x')
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin',
            email='parite@boulangerie-martin.fr',
            employee_count=5,
            user=cls.user,
        )
        License.objects.create(
            tenant=cls.tenant, plan='starter', seats=5, starts_at=date.today(),
        )

    def setUp(self):
        self.client.force_login(self.user)
        self.windows = self._installeur('windows')
        self.linux = self._installeur('linux')

    @override_settings(
        ALLOWED_HOSTS=['192.168.7.7'],
        SS_NODE_IMAGE='{registry}/ss-node:v2',
        REGISTRY_PORT='5000',
    )
    def _installeur(self, os_type):
        r = self.client.get(
            '/api/tenants/me/download/installer/' + os_type + '/',
            HTTP_HOST='192.168.7.7',
        )
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        return r.content.decode('utf-8')

    def _present_partout(self, libelle, motif_win, motif_lin):
        with self.subTest(element=libelle, os='windows'):
            self.assertIn(
                motif_win, self.windows,
                'Absent de l installeur Windows : ' + libelle,
            )
        with self.subTest(element=libelle, os='linux'):
            self.assertIn(
                motif_lin, self.linux,
                'Absent de l installeur Linux : ' + libelle,
            )

    # -- Etapes communes, dans le meme ordre ---------------------------------

    def test_les_deux_telechargent_le_compose_du_portail(self):
        """Source unique de verite : aucun des deux ne genere la pile.

        Un installeur qui recommence a ecrire son propre compose reintroduit la
        divergence que cette refonte supprime.
        """
        self._present_partout(
            'telechargement du docker-compose.yml',
            '/api/tenants/pme/compose/', '/api/tenants/pme/compose/',
        )
        for nom, contenu in (('windows', self.windows), ('linux', self.linux)):
            with self.subTest(os=nom):
                self.assertNotIn('image: containrrr/watchtower', contenu)
                self.assertNotIn('image: postgres:16', contenu)
                self.assertNotIn('driver: bridge', contenu)

    def test_les_deux_interrogent_cluster_status(self):
        """Le role est decide A L'INSTALLATION, pas au telechargement.

        Deux machines qui telechargent leur installeur avant de l'executer
        partiraient sinon toutes les deux en primaire : split-brain.
        """
        self._present_partout(
            'appel a cluster-status',
            '/api/devices/cluster-status/', '/api/devices/cluster-status/',
        )
        self._present_partout(
            'bascule en standby',
            'set "NODE_ROLE=standby"', 'NODE_ROLE="standby"',
        )
        self._present_partout(
            'bascule en passive',
            'set "NODE_MODE=passive"', 'NODE_MODE="passive"',
        )

    def test_les_deux_affichent_les_noeuds_et_demandent_lip_du_primaire(self):
        self._present_partout(
            'liste des noeuds enregistres',
            'Noeuds deja enregistres', 'déjà enregistrés',
        )
        self._present_partout(
            'saisie de l IP du primaire',
            'set /p "PRIMARY_IP=', 'read -rp',
        )

    def test_les_deux_derivent_les_trois_memes_mots_de_passe(self):
        """La replication PostgreSQL entre Windows et Fedora n'est possible que
        si REPLICATION_PASSWORD coincide exactement des deux cotes.

        Meme entree (le jeton d'enregistrement), meme suffixe, meme primitive
        (SHA-256), meme encodage (ASCII), meme sortie (hexadecimal minuscule
        tronque a 24 caracteres).
        """
        for suffixe in ('-pg', '-repl', '-pgadmin'):
            self._present_partout(
                'derivation ' + suffixe,
                "GetBytes('" + str(self.tenant.registration_token) + suffixe + "')",
                "printf '%s" + suffixe + "'",
            )
        self._present_partout(
            'SHA-256 hexadecimal minuscule tronque a 24',
            'ToLower().Substring(0,24)', 'sha256sum | cut -c1-24',
        )
        self._present_partout(
            'encodage ASCII', '[Text.Encoding]::ASCII', "printf '%s",
        )

    def test_les_deux_ecrivent_le_meme_env(self):
        """Le .env est le SEUL parametrage de la pile : une cle absente d'un
        cote donne deux piles differentes a partir du meme compose."""
        cles = [
            'NODE_ROLE', 'NODE_MODE', 'PG_USER', 'PG_PASSWORD', 'PG_DB',
            'REPLICATION_PASSWORD', 'PG_REPLICATION_SUBNET', 'PRIMARY_IP',
            'WEB_PORT', 'NODE_ADDR', 'NODE_HOSTNAME', 'RELAY_URL', 'SAAS_URL',
            'REGISTRATION_TOKEN', 'TENANT_ID', 'PGADMIN_EMAIL',
            'PGADMIN_PASSWORD',
        ]
        for cle in cles:
            self._present_partout('.env ' + cle, 'echo ' + cle + '=', cle + '=')

    def test_les_deux_replient_sur_larchive_si_le_pull_echoue(self):
        self._present_partout('pull depuis le registre', 'docker pull', 'docker pull')
        self._present_partout(
            'repli sur l archive',
            '/api/node-image/telecharger/', '/api/node-image/telecharger/',
        )
        self._present_partout('chargement de l archive', 'docker load', 'docker load')
        self._present_partout(
            'retiquetage de l image',
            'docker tag ss-node:v2', 'docker tag ss-node:v2',
        )

    def test_les_deux_declarent_le_registre_non_securise(self):
        self._present_partout(
            'insecure-registries', 'insecure-registries', 'insecure-registries',
        )

    def test_les_deux_ouvrent_les_trois_memes_ports(self):
        """5432 (replication), 5050 (pgAdmin), 9001 (noeud + interface web).

        L'installeur Windows n'ouvrait que 9001 : la Fedora ne pouvait donc pas
        repliquer depuis la machine Windows.
        """
        self._present_partout(
            'pare-feu PostgreSQL 5432', 'localport=5432', '5432/tcp',
        )
        self._present_partout(
            'pare-feu pgAdmin 5050', 'localport=5050', '5050/tcp',
        )
        self._present_partout(
            'pare-feu noeud 9001', 'localport=!WEB_PORT!', '${WEB_PORT}/tcp',
        )

    def test_le_port_web_est_le_meme_des_deux_cotes(self):
        self._present_partout(
            'WEB_PORT=9001', 'set "WEB_PORT=9001"', 'WEB_PORT=9001',
        )

    def test_les_deux_demarrent_la_pile_et_la_relancent_au_boot(self):
        self._present_partout(
            'demarrage de la pile', 'docker compose up -d', 'docker compose up -d',
        )
        self._present_partout(
            'demarrage automatique au boot',
            'schtasks /create', 'systemctl enable elbaraa-pme.service',
        )

    def test_les_deux_initialisent_la_cle_sur_le_primaire(self):
        self._present_partout(
            'ss-node init --first', 'ss-node init --first', 'ss-node init --first',
        )
