# Groupe D — Synthèse d'exécution (D-01 → D-06)

- **Date** : 2026-11-25
- **Base** : SQLite jetable `/tmp/opencode/d-tests/test-amane-d.sqlite3` (créée puis détruite).
- **Commande** : `python3 tools/run_d_tests.py`
- **Journal brut** : `docs/tests/artifacts-d01/execution-d01-d06.log`
- **Résultat global** : 11 tests, **7 échecs** (failures=7), 0 erreur.

| Code | Test(s) | Attendu | Obtenu | Verdict |
|---|---|---|---|---|
| D-01 | `D01UsurpationTests` | 403/409, machine A inchangée | **200**, machine A modifiée (hostname/node_addr) | **FAIL** |
| D-02 | `D02LicenceExpireeTests` | 403 `license_expired` ; inactive → 403 `no_license` | expirée → **201** ; inactive → 403 `no_license` | **FAIL** |
| D-03 | `D03SiegesConcurrentsTests` | 1×201 + 1×403, actifs=1 (à chaque fois) | **6 à 9 (sur 15)** itérations dévient : `[403,403]` actifs=0 | **FAIL** |
| D-04 | `D04ConfinementComposeTests` | réseau `internal`, app sans port, ss-node+HAProxy dessus, PG+etcd ailleurs | conforme | **PASS** |
| D-05 | `D05ManifesteDangereuxTests` | `ManifesteInvalide` pour 6 cas dangereux | **6/6 acceptés** | **FAIL** |
| D-06 | `D06RegistreTests` | 401/403/500 → 503 ; 302→169.254.169.254 bloqué | 401/403/500 → False (409) ; 302 suivie (SSRF) | **FAIL** |

## Détail des assertions

### D-01 (FAIL)
`POST /api/devices/register/` avec le jeton du tenant B et l'`installation_id`
de la machine du tenant A renvoie **200 authorized** (« Machine reconnue et
autorisée »), et l'enregistrement **écrase** `hostname` (`machine-a` →
`machine-pirate`) et `node_addr` (`192.168.1.10:9001` → `10.0.0.9:9001`) de la
machine A. `get_or_create(installation_id=…)` retrouve la machine existante sans
vérifier qu'elle appartient au tenant du jeton.

### D-02 (FAIL)
Licence `is_active=True` mais `expires_at` dans le passé → **201** (l'expiration
n'est pas vérifiée). Le sous-test `is_active=False` (prioritaire) donne bien 403
`no_license`.

### D-03 (FAIL)
`TransactionTestCase` (écritures réellement commitées), deux threads synchronisés
par `threading.Barrier`, base SQLite **fichier** partagée. Scénario répété 15
fois avec des `installation_id` uniques par itération. **6 à 9 (sur 15) itérations
dévient** : les deux requêtes reçoivent **403** et le compte actif tombe à **0** —
les deux threads voient `count=2 > 1` et désactivent chacun sa machine, rejetant
un enregistrement pourtant légitime. Le contrôle de sièges
(`get_or_create` → `count` → `update`) n'est pas atomique.

**Réserve** : SQLite sérialise les écritures au niveau fichier ; la course
Postgres (READ COMMITTED) pourrait produire l'inverse (`[201,201]`, 2 actifs).
Dans les deux cas l'invariant « 1×201 + 1×403 / actif=1 » n'est pas garanti.

### D-04 (PASS)
Compose d'un tenant avec application valide : `amane-application.internal=true`,
aucun `ports` sur `amane-application`, `ss-node` et `amane-haproxy` sur
`amane-application`, `amane-postgresql` et `amane-etcd` hors de ce réseau.

### D-05 (FAIL)
`valider()` accepte les 6 manifestes dangereux : volumes `/var/run/docker.sock`,
`/etc`, `/proc`, commande `sh -c curl…`, schémas `file:///etc/passwd` et
`gopher://`. Contrôle positif : le manifeste sain (`postgres://` + `/data`) est
bien accepté.

### D-06 (FAIL)
`image_presente()` renvoie `False` (→ 409 « image absente ») pour 401/403/500 au
lieu de `RegistreInjoignable` (→ 503). La vue `valider` rend **409** pour un
registre 401. Un `302` vers `http://169.254.169.254/…` est **suivi** (SSRF) :
aucune garde de redirection. Contrôles positifs 404→False et 200→True OK.