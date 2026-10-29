"""Catalogue des applications hébergées (refonte V3, étape 5, ADR-009)."""
import urllib.error
from datetime import date
from unittest import mock

from django.contrib.auth.models import User
from django.test import TestCase
from rest_framework import status

from licenses.models import License
from tenants.models import Tenant

from .models import Application
from .registre import RegistreInjoignable, image_presente

DIGEST = 'sha256:' + 'cd' * 32
MANIFESTE = {
    'image': 'editeurs/facturation@' + DIGEST,
    'commande': ['/app/serveur'],
    'port': 8080,
    'sante': '/sante',
    'base_de_donnees': {'variables': {'DATABASE_URL': 'postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}'}},
}
PRESENTE = 'applications.views.image_presente'


class BaseApplications(TestCase):

    @classmethod
    def setUpTestData(cls):
        cls.staff = User.objects.create_user(username='editeur', password='x', is_staff=True)
        cls.pme = User.objects.create_user(username='pme', password='x')
        cls.tenant = Tenant.objects.create(name='Garage Dupont', email='g@dupont.fr', employee_count=5, user=cls.pme)
        cls.licence = License.objects.create(tenant=cls.tenant, plan='pro', seats=3, starts_at=date.today())

    def setUp(self):
        self.client.force_login(self.staff)

    def creer(self, **modif):
        corps = {'nom': 'Facturation', 'editeur': 'Éditions Lumière', 'version': '1.0', 'manifeste': MANIFESTE}
        corps.update(modif)
        return self.client.post('/api/applications/', corps, content_type='application/json')

    def validee(self, **modif):
        app_id = self.creer(**modif).json()['id']
        with mock.patch(PRESENTE, return_value=True):
            self.client.post(f'/api/applications/{app_id}/valider/')
        return Application.objects.get(pk=app_id)


class CatalogueTests(BaseApplications):

    def test_creation_d_un_brouillon_au_manifeste_normalise(self):
        r = self.creer()
        self.assertEqual(r.status_code, status.HTTP_201_CREATED, r.content)
        d = r.json()
        self.assertEqual(d['statut'], 'brouillon')
        self.assertEqual(d['type_artefact'], 'docker')
        self.assertEqual(d['image'], MANIFESTE['image'])
        self.assertEqual(d['manifeste']['volumes'], [])
        self.assertEqual(d['licences_actives'], 0)

    def test_manifeste_invalide_refuse(self):
        r = self.creer(manifeste={**MANIFESTE, 'image': 'editeurs/facturation:latest'})
        self.assertEqual(r.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('manifeste', r.json())
        r = self.creer(manifeste={**MANIFESTE, 'type_artefact': 'exe'})
        self.assertEqual(r.status_code, status.HTTP_400_BAD_REQUEST)

    def test_nom_et_version_uniques(self):
        self.creer()
        self.assertEqual(self.creer().status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(self.creer(version='1.1').status_code, status.HTTP_201_CREATED)

    def test_reserve_aux_comptes_editeur(self):
        self.client.force_login(self.pme)
        self.assertEqual(self.client.get('/api/applications/').status_code, status.HTTP_403_FORBIDDEN)
        self.assertEqual(self.creer().status_code, status.HTTP_403_FORBIDDEN)
        self.client.logout()
        self.assertEqual(self.client.get('/api/applications/').status_code, status.HTTP_403_FORBIDDEN)


class ValidationTests(BaseApplications):

    def test_validation_si_l_image_est_dans_le_registre(self):
        app_id = self.creer().json()['id']
        with mock.patch(PRESENTE, return_value=False):
            r = self.client.post(f'/api/applications/{app_id}/valider/')
        self.assertEqual(r.status_code, status.HTTP_409_CONFLICT)
        with mock.patch(PRESENTE, side_effect=RegistreInjoignable()):
            r = self.client.post(f'/api/applications/{app_id}/valider/')
        self.assertEqual(r.status_code, status.HTTP_503_SERVICE_UNAVAILABLE)
        with mock.patch(PRESENTE, return_value=True) as verif:
            r = self.client.post(f'/api/applications/{app_id}/valider/')
        verif.assert_called_once_with(MANIFESTE['image'])
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        self.assertEqual(r.json()['statut'], 'validee')
        self.assertIsNotNone(r.json()['validee_le'])
        # Une seule validation.
        with mock.patch(PRESENTE, return_value=True):
            r = self.client.post(f'/api/applications/{app_id}/valider/')
        self.assertEqual(r.status_code, status.HTTP_409_CONFLICT)

    def test_manifeste_fige_apres_validation(self):
        app = self.validee()
        r = self.client.patch(f'/api/applications/{app.id}/',
                              {'manifeste': {**MANIFESTE, 'port': 9090}}, content_type='application/json')
        self.assertEqual(r.status_code, status.HTTP_400_BAD_REQUEST)
        # Le reste de la fiche reste modifiable, et renvoyer le même manifeste passe.
        r = self.client.patch(f'/api/applications/{app.id}/',
                              {'editeur': 'Lumière SAS', 'manifeste': MANIFESTE}, content_type='application/json')
        self.assertEqual(r.status_code, status.HTTP_200_OK, r.content)

    def test_brouillon_modifiable(self):
        app_id = self.creer().json()['id']
        r = self.client.patch(f'/api/applications/{app_id}/',
                              {'manifeste': {**MANIFESTE, 'port': 9090}}, content_type='application/json')
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        self.assertEqual(r.json()['manifeste']['port'], 9090)


class RattachementTests(BaseApplications):

    def rattacher(self, application_id):
        return self.client.patch(f'/api/licenses/{self.licence.id}/',
                                 {'application': application_id}, content_type='application/json')

    def test_seule_une_application_validee_se_rattache(self):
        brouillon = self.creer().json()['id']
        self.assertEqual(self.rattacher(brouillon).status_code, status.HTTP_400_BAD_REQUEST)
        app = self.validee(version='2.0')
        r = self.rattacher(str(app.id))
        self.assertEqual(r.status_code, status.HTTP_200_OK, r.content)
        self.assertEqual(r.json()['application_nom'], 'Facturation 2.0')
        self.assertEqual(self.client.get(f'/api/applications/{app.id}/').json()['licences_actives'], 1)
        # Détacher.
        self.assertIsNone(self.rattacher(None).json()['application'])

    def test_l_espace_pme_voit_l_application_hebergee(self):
        app = self.validee()
        self.rattacher(str(app.id))
        self.client.force_login(self.pme)
        d = self.client.get('/api/tenants/me/').json()
        self.assertEqual(d['application']['nom'], 'Facturation')
        self.assertEqual(d['application']['version'], '1.0')
        self.assertNotIn('manifeste', d['application'])
        self.assertEqual(d['active_license']['application_nom'], 'Facturation 1.0')

    def test_retrait(self):
        from tenants.services import application_du_tenant
        app = self.validee()
        self.rattacher(str(app.id))
        self.assertEqual(application_du_tenant(self.tenant)['image'], MANIFESTE['image'])
        r = self.client.post(f'/api/applications/{app.id}/retirer/')
        self.assertEqual(r.json()['statut'], 'retiree')
        self.assertIsNone(application_du_tenant(self.tenant))
        # Une application retirée ne se rattache plus.
        self.assertEqual(self.rattacher(str(app.id)).status_code, status.HTTP_400_BAD_REQUEST)

    def test_une_application_hebergee_ne_se_supprime_pas(self):
        app = self.validee()
        self.rattacher(str(app.id))
        self.assertEqual(self.client.delete(f'/api/applications/{app.id}/').status_code, status.HTTP_409_CONFLICT)
        self.rattacher(None)
        self.assertEqual(self.client.delete(f'/api/applications/{app.id}/').status_code, status.HTTP_204_NO_CONTENT)

    def test_licence_inactive_n_heberge_rien(self):
        from tenants.services import application_du_tenant
        app = self.validee()
        self.rattacher(str(app.id))
        License.objects.filter(pk=self.licence.pk).update(is_active=False)
        self.assertIsNone(application_du_tenant(self.tenant))


class RegistreTests(TestCase):

    IMAGE = 'editeurs/facturation@' + DIGEST

    def test_image_presente(self):
        reponse = mock.MagicMock(status=200)
        reponse.__enter__.return_value = reponse
        with mock.patch('urllib.request.urlopen', return_value=reponse) as ouvrir:
            self.assertTrue(image_presente(self.IMAGE))
        requete = ouvrir.call_args[0][0]
        self.assertEqual(requete.get_method(), 'HEAD')
        self.assertTrue(requete.full_url.endswith('/v2/editeurs/facturation/manifests/' + DIGEST))

    def test_image_absente(self):
        erreur = urllib.error.HTTPError('u', 404, 'absent', {}, None)
        with mock.patch('urllib.request.urlopen', side_effect=erreur):
            self.assertFalse(image_presente(self.IMAGE))

    def test_registre_injoignable(self):
        with mock.patch('urllib.request.urlopen', side_effect=urllib.error.URLError('refusé')):
            with self.assertRaises(RegistreInjoignable):
                image_presente(self.IMAGE)
