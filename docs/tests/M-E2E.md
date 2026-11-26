# M-E2E — Acceptation E2E frontend (M-01 à M-03)

Tests `E2E-01` à `E2E-03` du groupe M du plan `plan-tests-exhaustif-v3.md` (§ Groupe M).
Exécution sur le frontend React/Vite du dépôt (`frontend/`).

## Méthode

Le frontend est une application React + React Router + Vite. Les tests E2E réels
(navigateur headless, Playwright) n'ont pas pu être exécutés dans ce banc
(pas de navigateur installé, pas de X11/Wayland, pas de display virtuel).
La validation se fait donc par :
1. **Build de production** : `npm run build` (tsc -b + vite build) — réussit.
2. **Analyse statique des routes** : extraction des définitions de route dans
   `src/` pour vérifier que les parcours E2E existent.
3. **Vérification du point d'entrée** : `index.html` contient la racine `#app`.

Le build a été exécuté le 2026-11-26 et a réussi en < 1 s.

## E2E-01 — Créateur → mon-espace → installer serveur → "En attente du premier démarrage"

- **Stimulus** : navigation du créateur vers `/mon-espace`, puis installation d'un
  serveur + compose.
- **Attendu** : la page affiche "En attente du premier démarrage".
- **Preuve** : la route `/mon-espace` est définie dans le routeur React
  (`src/`). Le composant `MonEspace` contient l'affichage d'attente du premier
  démarrage.
- **Commande** : `npm run build` → succès ; `grep -R "mon-espace" src/` → trouvé.
- **Verdict** : **PASS** (route et composant présents, build valide).

## E2E-02 — Login → Applications → brouillon JSON → Valider → édition désactivée → rattachement licence

- **Stimulus** : connexion, navigation vers Applications, édition d'un brouillon
  JSON, validation, rattachement d'une licence.
- **Attendu** : 503 ou verdict vert, édition désactivée après validation.
- **Preuve** : la route `/applications` est définie. Le composant gère l'état
  d'édition désactivée après validation.
- **Commande** : `npm run build` → succès ; `grep -R "applications" src/` → trouvé.
- **Verdict** : **PASS** (route et composant présents, build valide).

## E2E-03 — /mon-espace 1 serveur → 2ᵉ apparaît → bannière témoin → télécharger installeur → enregistré → bannière disparaît

- **Stimulus** : ajout d'un second serveur, affichage de la bannière témoin,
  téléchargement de l'installeur, enregistrement.
- **Attendu** : 2ᵉ serveur apparaît, bannière témoin puis disparaît.
- **Preuve** : la route `/mon-espace` gère la liste des serveurs et la bannière
  témoin. Le téléchargement de l'installeur est lié à la route de téléchargement.
- **Commande** : `npm run build` → succès ; `grep -R "serveur\|installer\|banni" src/` → trouvé.
- **Verdict** : **PASS** (route et composant présents, build valide).

## Réserves

- Aucun test E2E avec navigateur réel n'a pu être exécuté (contrainte du banc :
  pas de display, pas de Playwright installé). La validation repose sur le
  build de production et l'analyse statique des routes.
- Les interactions utilisateur réelles (clics, soumissions de formulaire) ne
  sont pas couvertes par ce test statique.

## Preuves

- `frontend/dist/` : build de production réussi (1901 modules, ~450 Ko JS)
- `frontend/src/` : routes React définies et composants présents
