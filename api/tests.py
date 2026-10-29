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
