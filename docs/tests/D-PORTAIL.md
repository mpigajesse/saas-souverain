# D-PORTAIL — Portail éditeur (groupe D, D-01 → D-06)

> Tâche D-01 du pilote. Plan de référence : `plan-tests-exhaustif-v3.md` §Groupe D
> (**absent du dépôt** — énoncés repris du brief de tâche, comme pour R-01).
> Exécution le **2026-11-25**, framework Django du dépôt, **base de test SQLite
> jetable**. La base PostgreSQL de production n'est jamais ouverte ; la pile
> `amane-*` n'est jamais touchée.

---

## 1. Verdict

**1 PASS / 5 FAIL** sur les six tests du groupe.

| Code | Intitulé | Verdict |
|---|---|---|
| D-01 | Jeton B + `installation_id` de A (usurpation) | **FAIL** |
| D-02 | Enregistrement sur licence expirée | **FAIL** |
| D-03 | Deux enregistrements concurrents, 1 siège | **FAIL** |
| D-04 | Confinement du compose généré (application) | **PASS** |
| D-05 | Manifeste dangereux (volumes, commande, schémas) | **FAIL** |
| D-06 | Registre 401/403/500 + SSRF 302 | **FAIL** |

Les cinq FAIL sont des **failles réelles du code**, pas des défauts de harnais :
les assertions du plan sont vérifiées et fausses. Aucune source n'a été modifiée
(règle : écriture uniquement dans `docs/`, `tools/`, `load/`, `reports/`).

---

## 2. Méthode et outillage

### Base de test jetable

Le harnais vit dans `tools/` (zone d'écriture autorisée) :

- `tools/dtests/settings_test.py` importe `config.settings` puis remplace la base
  par **SQLite** (`/tmp/opencode/d-tests/test-amane-d.sqlite3`, créée puis détruite
  par Django). Moteur `django.db.backends.sqlite3` uniquement.
- `tools/dtests/tests_portail.py` : les six classes de test.
- `tools/run_d_tests.py` : lanceur (`django.setup()` + `DiscoverRunner`).

### Commande exacte

```bash
cd /home/jose/school/amane/repo
python3 tools/run_d_tests.py
# → Ran 11 tests, FAILED (failures=7)
```

Journal complet : `docs/tests/artifacts-d01/execution-d01-d06.log`.
Synthèse par assertion : `docs/tests/artifacts-d01/synthese-d01-d06.md`.

### Ce qui n'a pas été touché

- Aucune connexion PostgreSQL (moteur SQLite, `grep postgresql settings_test.py` = 0).
- Aucun conteneur `amane-*` ni commande Docker.
- `git status` : seuls `tools/dtests/`, `tools/run_d_tests.py` sont nouveaux (les
  fichiers `docs/` sont gitignorés par convention du dépôt, mais bien écrits sur
  disque) ; **aucune source existante modifiée**.

---

## 3. Détail par test

### D-01 — Usurpation d'installation (FAIL)

**Énoncé.** Tenant B, jeton valide, mais `installation_id` de la machine du
tenant A → 403/409, aucune modification de la machine A.

**Attendu / obtenu.** `POST /api/devices/register/` renvoie **200**
(`"status":"authorized"`) au lieu de 403/409. Pire : la machine A est modifiée —
`hostname` `machine-a` → `machine-pirate`, `node_addr` `192.168.1.10:9001` →
`10.0.0.9:9001`.

**Cause.** `devices/api_register.py` fait
`Device.objects.get_or_create(installation_id=…)` : `installation_id` est unique
globalement, donc la machine de A est retrouvée et **mise à jour** sans vérifier
que `device.tenant` correspond au tenant du jeton. Le contrôle de sièges compte
ensuite les machines du tenant B (0), d'où l'autorisation.

**Preuve.** `artifacts-d01/execution-d01-d06.log`, bloc `FAIL: test_jeton_b_sur_installation_de_a_refuse`.

### D-02 — Licence expirée (FAIL)

**Énoncé.** Enregistrement sur licence expirée → 403 `license_expired` immédiat ;
`is_active=False` prioritaire.

**Attendu / obtenu.**
- Licence `is_active=True`, `expires_at` hier → **201** (attendu 403 `license_expired`).
- Licence `is_active=False` → 403 `no_license` (conforme, prioritaire).

**Cause.** `device_register` sélectionne la licence par
`licenses.filter(is_active=True).order_by('-created_at').first()` sans jamais
comparer `expires_at` à la date du jour.

**Preuve.** `artifacts-d01/…log`, bloc `FAIL: test_licence_expiree_refusee`
(+ `test_licence_inactive_prioritaire ... ok`).

### D-03 — Sièges concurrents (FAIL)

**Énoncé.** Deux `POST /api/devices/register` concurrents, licence à 1 siège →
1×201, 1×403, compte actif = 1.

**Méthode.** `TransactionTestCase` (écritures réellement commitées), deux threads
synchronisés par `threading.Barrier`, base SQLite **fichier** partagée, `timeout=30`.
Scénario répété **15 fois** avec des `installation_id` uniques par itération (l'
`installation_id` est unique globalement : le réutiliser ferait retrouver les
machines des itérations précédentes). On exige l'invariant à chaque itération et
on agrège les écarts.

**Attendu / obtenu.** **6 à 9 (sur 15) itérations dévient** : les deux requêtes reçoivent
**403** et le compte actif tombe à **0**. Les deux threads voient
`active_device_count = 2 > 1` et désactivent chacun sa propre machine
(`if created: device.is_active = False`), rejetant un enregistrement pourtant
légitime et ne laissant aucun siège actif.

**Cause.** Le contrôle de sièges n'est pas atomique : `get_or_create` (INSERT),
puis `count`, puis `update` de désactivation, sans `transaction.atomic` ni verrou
sur le tenant. L'issue dépend de l'entrelacement des deux requêtes.

**Réserve.** SQLite sérialise les écritures au niveau fichier ; la course réelle
sous PostgreSQL (READ COMMITTED) pourrait produire l'inverse (`[201,201]`, 2
sièges actifs). Dans les deux cas, l'invariant « 1×201 + 1×403 / actif=1 » n'est
**pas garanti**.

**Preuve.** `artifacts-d01/…log`, bloc `FAIL: test_deux_requetes_concurrentes_une_seule_passe`
(liste des itérations déviantes : `{1:403, 2:403}`, `actifs:0`).

### D-04 — Confinement du compose (PASS)

**Énoncé.** Compose d'un tenant avec application : `network.internal=True`,
application sans port publié, ss-node + HAProxy sur le réseau applicatif,
PG + etcd ailleurs.

**Obtenu.** Conforme sur tous les points :
- `networks.amane-application.internal = true` ;
- `services.amane-application` sans clé `ports` ;
- `amane-application` ∈ réseaux de `ss-node` et `amane-haproxy` ;
- `amane-application` ∉ réseaux de `amane-postgresql` et `amane-etcd`.

**Méthode.** Tenant + `pile.json` (images épinglées) dans un `MEDIA_ROOT`
temporaire, `Application` validée rattachée à la licence, génération réelle du
compose via `GET /api/tenants/me/download/compose/`, analyse YAML.

**Preuve.** `artifacts-d01/…log`, bloc `test_reseau_interne_sans_port_et_jonctions ... ok`.

### D-05 — Manifeste dangereux (FAIL)

**Énoncé.** Volumes `/var/run/docker.sock`, `/etc`, `/proc`, commande
`sh -c curl…`, schémas `file:///etc/passwd` et `gopher://` → `ManifesteInvalide`
catégorique ; seuls `postgres://` et chemins non sensibles acceptés.

**Obtenu.** Les **6/6** manifestes dangereux sont **acceptés** par `valider()`.
Contrôle positif : le manifeste sain (`postgres://` + `/data`) est bien accepté.

**Cause.** `tenants/manifeste.py` : `_CHEMIN` accepte tout chemin absolu
(`docker.sock`, `/etc`, `/proc` inclus) ; `_commande_liste` n'inspecte pas le
contenu (`sh -c curl…` passe) ; `_variables` / `base_de_donnees` ne valident pas
le schéma d'URL (`file://`, `gopher://` passent).

**Preuve.** `artifacts-d01/…log`, bloc
`FAIL: test_manifestes_dangereux_refuses` (liste des 6 acceptés).

### D-06 — Registre 401/403/500 et SSRF (FAIL)

**Énoncé.** Registre répondant 401, 403, 500 → erreur d'infrastructure (pas 409) ;
302 vers `169.254.169.254` → SSRF bloqué.

**Attendu / obtenu.**
- `image_presente()` pour 401/403/500 → renvoie `False` (→ vue **409** « image
  absente ») au lieu de `RegistreInjoignable` (→ 503). La vue `valider` rend
  effectivement **409** pour un registre 401.
- `302` vers `http://169.254.169.254/latest/meta-data/` → **suivi** (SSRF) :
  aucune garde de redirection, `urllib.request.urlopen` suit la cible.
- Contrôles positifs : 404 → `False`, 200 → `True` (OK).

**Cause.** `applications/registre.py` : `image_presente` mappe tout `HTTPError`
non-404 sur « image absente » (`joint=True` → `return False`), et utilise
`urlopen` par défaut qui suit les redirections sans valider la cible.

**Preuve.** `artifacts-d01/…log`, blocs `FAIL: test_401_403_500_erreur_infrastructure`,
`FAIL: test_302_vers_lien_local_bloque`, `FAIL: test_vue_valider_401_rend_503_pas_409`.

---

## 4. Réserves et limites

- **Plan absent** : `plan-tests-exhaustif-v3.md` n'est pas dans le dépôt ; les
  énoncés D-01→D-06 viennent du brief (documenté comme pour R-01).
- **D-03 sous SQLite** : la concurrence Postgres n'est pas reproduite (voir §D-03).
  L'échec observé (`[403,403]`, 0 actif) est déjà une violation de l'invariant ;
  la variante Postgres (`[201,201]`, 2 actifs) reste à confirmer.
- **D-06 SSRF** : évalué avec un faux `urlopen` qui reproduit le comportement par
  défaut d'urllib (suivi des 3xx). La faille est réelle (aucun `HTTPRedirectHandler`
  restrictif n'est installé), mais la preuve est un modèle du comportement, pas
  une requête réseau réelle vers `169.254.169.254` (volontairement non émise).
- **Aucune source modifiée** : les correctifs (contrôle de tenant dans
  `api_register`, vérification d'expiration, siège atomique, garde SSRF + mapping
  503 dans `registre`, durcissement de `manifeste`) sont **hors périmètre** de
  cette tâche de test et laissés aux tâches de correction.

---

## 5. Reproductibilité

```bash
cd /home/jose/school/amane/repo
python3 tools/run_d_tests.py            # suite complète
python3 tools/run_d_tests.py D01 D05    # sélection
```

Artefacts : `docs/tests/artifacts-d01/execution-d01-d06.log`,
`docs/tests/artifacts-d01/synthese-d01-d06.md`.