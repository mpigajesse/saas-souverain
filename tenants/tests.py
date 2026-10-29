"""Pile PME V3 générée par le portail — refonte V3, ADR-001, ADR-004, ADR-005.

Ces tests verrouillent les promesses de ce que le portail sert à une PME :

- images épinglées par digest, et refus de générer la pile sans elles ;
- aucun secret connu du portail (plus de mot de passe dérivé du jeton) ;
- bascule du nœud Rust neutralisée : etcd + Patroni décident seuls ;
- deux types de machine, serveur et témoin, servis par une définition unique ;
- installeurs Windows et Linux qui déroulent les mêmes étapes, sans question
  sur le primaire.
"""
import json
import re
import tempfile
from datetime import date
from pathlib import Path

import yaml
from django.contrib.auth.models import User
from django.test import TestCase, override_settings
from rest_framework import status

from devices.models import Device
from licenses.models import License
from tenants.models import Tenant
from tenants.pile import IMAGES_PILE

HOTE = '192.168.7.7'
DIGEST = 'sha256:' + 'ab' * 32
MANIFESTE = {
    'publie_le': '2026-10-29T00:00:00',
    'images': {nom: f'amane/{nom}@{DIGEST}' for nom in IMAGES_PILE},
}
REGLAGES = dict(ALLOWED_HOSTS=[HOTE], REGISTRY_PORT='5000')


def ecrire_manifeste(media_root: str, contenu) -> None:
    dossier = Path(media_root) / 'node-images'
    dossier.mkdir(parents=True, exist_ok=True)
    texte = contenu if isinstance(contenu, str) else json.dumps(contenu)
    (dossier / 'pile.json').write_text(texte, encoding='utf-8')


def corps(response) -> str:
    if response.streaming:
        return b''.join(response.streaming_content).decode('utf-8')
    return response.content.decode('utf-8')


class BasePile(TestCase):
    """Tenant PME connecté + manifeste d'images publié dans un MEDIA_ROOT
    temporaire."""

    @classmethod
    def setUpTestData(cls):
        cls.user = User.objects.create_user(username='pme-v3', password='x')
        cls.tenant = Tenant.objects.create(
            name='Boulangerie Martin', email='v3@boulangerie-martin.fr',
            employee_count=5, user=cls.user,
        )
        License.objects.create(tenant=cls.tenant, plan='starter', seats=2, starts_at=date.today())

    def setUp(self):
        self.media = tempfile.TemporaryDirectory()
        self.addCleanup(self.media.cleanup)
        ecrire_manifeste(self.media.name, MANIFESTE)
        reglages = override_settings(MEDIA_ROOT=self.media.name, **REGLAGES)
        reglages.enable()
        self.addCleanup(reglages.disable)
        self.client.force_login(self.user)

    def compose(self, type_machine='serveur', par_jeton=False):
        if par_jeton:
            r = self.client.get('/api/tenants/pme/compose/', {
                'token': str(self.tenant.registration_token), 'type': type_machine,
            }, HTTP_HOST=HOTE)
        else:
            r = self.client.get('/api/tenants/me/download/compose/', {'type': type_machine}, HTTP_HOST=HOTE)
        return r

    def installeur(self, os_type, type_machine='serveur'):
        return self.client.get(
            f'/api/tenants/me/download/installer/{os_type}/', {'type': type_machine}, HTTP_HOST=HOTE,
        )


class ImagesEpingleesTests(BasePile):
    """ADR-005, décision 3 : la PME exécute exactement ce qui a été validé."""

    def test_toutes_les_images_sont_epinglees_par_digest(self):
        for type_machine in ('serveur', 'temoin'):
            texte = corps(self.compose(type_machine))
            images = re.findall(r'^\s+image:\s*(\S+)', texte, re.M)
            self.assertTrue(images, type_machine)
            for image in images:
                self.assertRegex(image, rf'^{HOTE}:5000/amane/[a-z]+@{DIGEST}$', type_machine)

    def test_sans_manifeste_la_pile_nest_pas_generee(self):
        (Path(self.media.name) / 'node-images' / 'pile.json').unlink()
        self.assertEqual(self.compose().status_code, status.HTTP_503_SERVICE_UNAVAILABLE)
        self.assertEqual(self.compose(par_jeton=True).status_code, status.HTTP_503_SERVICE_UNAVAILABLE)

    def test_une_image_referencee_par_tag_est_refusee(self):
        manifeste = json.loads(json.dumps(MANIFESTE))
        manifeste['images']['postgresql'] = 'amane/spilo-17:4.0-p3'
        ecrire_manifeste(self.media.name, manifeste)
        self.assertEqual(self.compose().status_code, status.HTTP_503_SERVICE_UNAVAILABLE)

    def test_manifeste_illisible(self):
        ecrire_manifeste(self.media.name, '{pas du json')
        self.assertEqual(self.compose().status_code, status.HTTP_503_SERVICE_UNAVAILABLE)


class ComposeServeurTests(BasePile):

    def setUp(self):
        super().setUp()
        r = self.compose()
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        self.texte = corps(r)
        self.doc = yaml.safe_load(self.texte)

    def test_services_de_la_pile(self):
        self.assertEqual(sorted(self.doc['services']), [
            'amane-etcd', 'amane-haproxy', 'amane-init', 'amane-orchestrateur',
            'amane-postgresql', 'pgadmin', 'ss-node',
        ])
        self.assertNotIn('watchtower', self.texte)

    def test_aucun_secret_connu_du_portail(self):
        """ADR-005, décision 4 : plus aucun mot de passe dérivé du jeton."""
        for interdit in (r'sha256sum', r'PG_PASSWORD', r'REPLICATION_PASSWORD', r'PGADMIN_PASSWORD:'):
            self.assertNotRegex(self.texte, interdit)
        self.assertNotIn(str(self.tenant.registration_token), self.texte)
        # Les conteneurs lisent les secrets générés sur le réseau de la PME.
        for service in ('amane-postgresql', 'ss-node', 'pgadmin'):
            self.assertIn('/amane/secrets/postgresql.env', json.dumps(self.doc['services'][service]))

    def test_le_noeud_lit_patroni_et_ecrit_par_haproxy(self):
        """Étape 2c : le nœud ne décide d'aucune bascule, il lit son rôle dans
        Patroni et écrit sur le primaire du moment par HAProxy."""
        env = self.doc['services']['ss-node']['environment']
        self.assertEqual(env['PATRONI_URL'], 'http://amane-postgresql:8008')
        self.assertNotIn('CLUSTER_PEERS', env)
        self.assertIn('amane:$${AMANE_PG_PASSWORD}@amane-haproxy:5000/amane', self.texte)

    def test_patroni_annonce_lip_lan_et_replique_en_synchrone(self):
        env = self.doc['services']['amane-postgresql']['environment']
        self.assertEqual(env['PG_CONNECT_ADDRESS'], '${NODE_IP}:5432')
        self.assertEqual(env['RESTAPI_CONNECT_ADDRESS'], '${NODE_IP}:8008')
        dcs = yaml.safe_load(env['SPILO_CONFIGURATION'])['bootstrap']['dcs']
        self.assertTrue(dcs['synchronous_mode'])
        self.assertEqual(dcs['postgresql']['parameters']['max_wal_senders'], 5)
        self.assertNotIn('synchronous_mode_stop_on_limit', self.texte)

    def test_membre_patroni_nomme_comme_la_machine(self):
        """Spilo nomme le membre d'après le nom d'hôte du conteneur : sans ce
        nom, l'orchestrateur ne reconnaîtrait pas son nœud local (constaté à
        l'essai sur trois machines de l'étape 2b)."""
        pg = self.doc['services']['amane-postgresql']
        self.assertEqual(pg['hostname'], '${NODE_HOSTNAME}')
        orch = self.doc['services']['amane-orchestrateur']
        self.assertEqual(orch['environment']['AMANE_NODE_ID'], '${NODE_HOSTNAME}')

    def test_orchestrateur_decouvre_et_gere_haproxy(self):
        cmd = self.doc['services']['amane-orchestrateur']['command']
        for option in ('-supervisor', '-decouvrir', '-secrets=/amane/secrets/postgresql.env', '-haproxy-dossier=/amane/haproxy'):
            self.assertIn(option, cmd)
        init = self.doc['services']['amane-init']['command']
        self.assertEqual(init[0], 'init')
        self.assertIn('-portail=${SAAS_URL}', init)

    def test_etcd_et_postgresql_joignables_par_les_autres_machines(self):
        self.assertEqual(self.doc['services']['amane-etcd']['ports'], ['2379:2379', '2380:2380'])
        self.assertEqual(self.doc['services']['amane-postgresql']['ports'], ['5432:5432', '8008:8008'])
        self.assertNotIn('ports', self.doc['services']['amane-haproxy'])

    def test_voie_par_jeton_identique_a_la_voie_par_session(self):
        def nettoyer(texte):
            return [l for l in texte.splitlines() if not l.startswith('# Générée le')]
        self.assertEqual(nettoyer(corps(self.compose(par_jeton=True))), nettoyer(self.texte))


class ComposeTemoinTests(BasePile):

    def test_le_temoin_ne_porte_quun_membre_etcd(self):
        r = self.compose('temoin')
        self.assertEqual(r.status_code, status.HTTP_200_OK)
        doc = yaml.safe_load(corps(r))
        self.assertEqual(sorted(doc['services']), ['amane-etcd', 'amane-init', 'amane-temoin'])
        self.assertEqual(doc['services']['amane-temoin']['command'][0], 'temoin')
        self.assertNotIn('postgresql', corps(r))

    def test_type_invalide(self):
        self.assertEqual(self.compose('arbitre').status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(self.compose('arbitre', par_jeton=True).status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(self.installeur('linux', 'arbitre').status_code, status.HTTP_400_BAD_REQUEST)


class InstalleursTests(BasePile):
    """Windows et Linux déroulent les mêmes étapes (V3)."""

    CLES_ENV = ('NODE_IP', 'NODE_HOSTNAME', 'NODE_MODE', 'WEB_PORT', 'SAAS_URL',
                'RELAY_URL', 'REGISTRATION_TOKEN', 'TENANT_ID', 'PGADMIN_EMAIL')

    def textes(self, type_machine='serveur'):
        out = {}
        for os_type in ('linux', 'windows'):
            r = self.installeur(os_type, type_machine)
            self.assertEqual(r.status_code, status.HTTP_200_OK)
            out[os_type] = (corps(r), r['Content-Disposition'])
        return out

    def test_meme_env_des_deux_cotes(self):
        for os_type, (texte, _) in self.textes().items():
            for cle in self.CLES_ENV:
                self.assertRegex(texte, rf'(?m)^(echo )?{cle}=', f'{os_type} : {cle}')

    def test_plus_aucune_question_ni_derivation(self):
        for type_machine in ('serveur', 'temoin'):
            for os_type, (texte, _) in self.textes(type_machine).items():
                for interdit in ('PRIMARY_IP', 'sha256sum', 'SHA256', 'docker load', 'read -rp', 'set /p'):
                    self.assertNotIn(interdit, texte, f'{os_type}/{type_machine} : {interdit}')

    def test_pile_telechargee_avec_le_type_de_machine(self):
        for type_machine in ('serveur', 'temoin'):
            for os_type, (texte, disposition) in self.textes(type_machine).items():
                self.assertIn('/api/tenants/pme/compose/?token=', texte)
                self.assertIn('docker compose pull', texte)
                self.assertIn(f'TYPE_MACHINE={type_machine}', texte.replace('"', '').replace('set ', ''))
                if type_machine == 'temoin':
                    self.assertIn('-temoin.', disposition, os_type)

    def test_pare_feu(self):
        serveur = self.textes('serveur')
        self.assertIn('2379 2380 5432 8008', serveur['linux'][0])
        self.assertIn('localport=2379,2380', serveur['windows'][0])
        self.assertIn('localport=5432,8008', serveur['windows'][0])

    def test_ancienne_pile_arretee_sans_perdre_ses_volumes(self):
        for os_type, (texte, _) in self.textes().items():
            self.assertIn('ElBaraaConsult' if os_type == 'windows' else '/opt/elbaraa-pme', texte)
            self.assertNotIn('down -v', texte, os_type)

    def test_premier_serveur_detecte_par_cluster_status(self):
        for os_type, (texte, _) in self.textes().items():
            self.assertIn('/api/devices/cluster-status/', texte, os_type)
            self.assertIn('node_kind', texte, os_type)

    def test_registre_non_securise_et_demarrage_au_boot(self):
        textes = self.textes()
        self.assertIn('insecure-registries', textes['linux'][0])
        self.assertIn('insecure-registries', textes['windows'][0])
        self.assertIn('systemctl enable amane-pme.service', textes['linux'][0])
        self.assertIn('schtasks /create /tn "AmanePME"', textes['windows'][0])

    def test_installeur_windows_en_ascii_pur(self):
        for type_machine in ('serveur', 'temoin'):
            texte = self.textes(type_machine)['windows'][0]
            self.assertEqual(texte, texte.encode('ascii', 'ignore').decode('ascii'), type_machine)


class EspacePmeTests(BasePile):

    def ajouter(self, kind, n):
        for i in range(n):
            Device.objects.create(
                tenant=self.tenant, installation_id=f'00000000-0000-4000-8000-{kind[:1]}{i:011d}'.replace('s', '1').replace('t', '2'),
                hostname=f'{kind}-{i}', node_kind=kind, node_addr=f'192.168.1.{10 + i}:9001',
            )

    def moi(self):
        return self.client.get('/api/tenants/me/', HTTP_HOST=HOTE).json()

    def test_mot_de_passe_pgadmin_inconnu_du_portail(self):
        donnees = self.moi()
        self.assertNotIn('password', donnees['pgadmin'])
        self.assertTrue(donnees['pgadmin']['email'].startswith('admin@pme'))

    def test_temoin_conseille_a_deux_serveurs(self):
        self.ajouter('serveur', 2)
        self.assertTrue(self.moi()['besoin_temoin'])
        self.ajouter('temoin', 1)
        self.assertFalse(self.moi()['besoin_temoin'])

    def test_temoin_inutile_a_trois_serveurs(self):
        self.ajouter('serveur', 3)
        self.assertFalse(self.moi()['besoin_temoin'])

    def test_liens_dinstallation_du_temoin(self):
        liens = self.client.get('/api/tenants/me/onboarding/', HTTP_HOST=HOTE).json()['downloads']
        self.assertTrue(liens['installer_temoin_linux'].endswith('?type=temoin'))
        self.assertTrue(liens['installer_temoin_windows'].endswith('?type=temoin'))


class EnregistrementTemoinTests(TestCase):
    """Un témoin s'enregistre comme une machine mais ne consomme pas de poste."""

    URL = '/api/devices/register/'

    @classmethod
    def setUpTestData(cls):
        cls.tenant = Tenant.objects.create(name='PME', email='temoin@pme.fr', employee_count=1)
        License.objects.create(tenant=cls.tenant, plan='starter', seats=1, starts_at=date.today())

    def enregistrer(self, installation_id, kind=None):
        corps_json = {'tenant_token': str(self.tenant.registration_token), 'installation_id': installation_id,
                      'node_addr': '192.168.1.30:2379'}
        if kind:
            corps_json['node_kind'] = kind
        return self.client.post(self.URL, corps_json, content_type='application/json')

    def test_le_temoin_ne_consomme_pas_de_poste(self):
        self.assertEqual(self.enregistrer('11111111-1111-4111-8111-111111111111').status_code, 201)
        r = self.enregistrer('22222222-2222-4222-8222-222222222222', 'temoin')
        self.assertEqual(r.status_code, 201, r.content)
        self.assertTrue(self.tenant.licence_status['ok'])
        self.assertEqual(self.tenant.licence_status['used'], 1)
        # Un second SERVEUR dépasse bien la licence.
        self.assertEqual(self.enregistrer('33333333-3333-4333-8333-333333333333').status_code, 403)

    def test_type_invalide_refuse(self):
        self.assertEqual(self.enregistrer('44444444-4444-4444-8444-444444444444', 'arbitre').status_code, 400)

    def test_cluster_status_expose_le_type(self):
        self.enregistrer('55555555-5555-4555-8555-555555555555', 'temoin')
        noeuds = self.client.get('/api/devices/cluster-status/', {'token': str(self.tenant.registration_token)}).json()
        self.assertEqual(noeuds['active_nodes'][0]['node_kind'], 'temoin')
