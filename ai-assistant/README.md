# AI Assistant — Agents de supervision pour le cluster Amane

Module **Mission C (AI)** : deux agents d'aide à la supervision d'un cluster distribué
Amane, adossés à un **modèle IA 100 % local** (Ollama). Le modèle ne sort jamais du
nœud — aucune donnée n'est envoyée vers un service cloud (souveraineté).

> **Cette branche est dédiée au code agent IA uniquement.**
> `ai-assistant/` (module Go) + `orchestrator-go/clusterreader/` (interface read-only exposée par Mission C).
> Tout le reste du monorepo est ignoré (voir `.gitignore`).

---

## Table des matières

- [Rôle du module](#rôle-du-module)
- [Architecture](#architecture)
- [Les deux agents](#les-deux-agents)
  - [Sentinelle — diagnostic (lecture seule)](#sentinelle--diagnostic-lecture-seule)
  - [Copilote — actions proposées sous contrôle](#copilote--actions-proposées-sous-contrôle)
- [Modèle IA local (Ollama)](#modèle-ia-local-ollama)
- [Sécurité & règles non négociables](#sécurité--règles-non-négociables)
- [Jalons de validation (M1 → M4)](#jalons-de-validation-m1--m4)
- [Installation & mise en route](#installation--mise-en-route)
- [Tests](#tests)
- [Variables d'environnement](#variables-denvironnement)

---

## Rôle du module

Fournir des **agents IA de supervision** à un propriétaire de PME qui n'a pas de
service IT. Les agents expliquent **en français, sans jargon technique**, ce qui se
passe dans le cluster distribuée Amane (etcd/Patroni/PostgreSQL), et proposent
parfois des actions **sûres**, toujours soumises à une **liste blanche stricte** et à
la confirmation humaine.

Architecture de principe :

```
┌─────────────┐   ClusterReader (read-only)   ┌──────────────────┐
│  AMANE POD  │  ── gRPC / interface Go ───▶ │  ai-assistant    │
│  (C Cluster)│                               │  Sentinelle      │
│ etcd+Patroni│                               │  Copilote        │
│ PostgreSQL  │                               └────────┬─────────┘
└─────────────┘                                        │  prompt
                                                        ▼
                                        ┌──────────────────────────┐
                                        │  Ollama (local)          │
                                        │  phi3:mini (défaut)      │
                                        │  127.0.0.1:11434         │
                                        └──────────────────────────┘
```

Point clé : le module IA **n'accède jamais directement** à etcd, PostgreSQL ou aux
RPC internes. Il passe **uniquement** par l'interface `ClusterReader`
(`orchestrator-go/clusterreader`), read-only par construction.

---

## Architecture

```
ai-assistant/
├── agents/
│   ├── sentinelle.go      # Agent diagnostic (lecture seule)
│   ├── copilote.go        # Agent d'action (whitelist + confirmation)
│   └── llm.go             # Type LLMFunc + construit les prompts
├── llm/
│   └── llm.go             # Client HTTP Ollama (/api/generate, stream off)
├── whitelist/
│   └── whitelist.go       # Liste blanche stricte des actions autorisées
├── journal/
│   └── journal.go         # Journalisation des interactions IA (traçabilité)
├── context/
│   └── context.go         # Contexte IA (état cluster visible par le modèle)
├── simulator/
│   └── failure.go         # Simulateur de pannes (tests M3/M4)
├── fixtures/
│   └── frequent_questions.json  # Questions fréquentes (coût M1)
├── m1_test.go … m4_test.go      # Jalons M1–M4
└── stub_llm_test.go             # Stub LLM déterministe (tests sans réseau)
```

**Dépendance externe** : `orchestrator-go/clusterreader` (via le `replace` du
`go.mod`). C'est la seule porte d'entrée du module IA vers l'état du cluster.

---

## Les deux agents

### Sentinelle — diagnostic (lecture seule)

Explique l'état du cluster en français simple. **Ne prend jamais d'action.**

| Méthode | Description |
|---|---|
| `NewSentinelle(logger)` | Constructeur (résout le modèle depuis l'env) |
| `Diagnose(ctx, reader)` | Produit un diagnostic en français simple |
| `SetLLM(fn)` / `SetModel(m)` | Injection (tests) / changement de modèle |

**Comportement** :
- Construit un prompt structuré : état (leader, quorum, primary) + dernières entrées
  de journal (séquences + types d'opération uniquement).
- Appelle le modèle local (Ollama).
- Si le modèle est **injoignable** ou la réponse **vide** → réponse **générique sûre**
  (jamais d'erreur brute).
- Prompt interdit au modèle : divulguer des clés (AK/DEK/KEK), suggérer toute action
  critique non validée.

### Copilote — actions proposées sous contrôle

Propose une action à partir de l'intention utilisateur, **valide stricte contre la
liste blanche**, demande **confirmation humaine** (si requise), puis exécute. Il **ne
décide jamais seul**.

| Méthode | Description |
|---|---|
| `NewCopilote(logger, dryRun)` | Constructeur (dryRun bloque toute exécution) |
| `ProposeAction(ctx, reader, intent)` | Pipeline complet : LLM → whitelist → confirmation → exécution |
| `SetConfirmFunc(fn)` | Remplace la confirmation (stdin par défaut) |
| `SetJournal(writer, nodeID)` | Active la journalisation de traçabilité (M4) |
| `SetDryRun(bool)` / `IsDryRun()` | Mode test : propose mais n'exécute jamais |

**Pipeline `ProposeAction`** :

```
Intention utilisateur
   │
   ▼
Prompt → LLM local (Ollama)  →  réponse JSON { action, args }
   │
   ▼  (la sortie LLM est NON fiable par construction)
Validation stricte whitelist  ── échoue ──▶ rejet (action non autorisée)
   │  OK
   ▼
dry-run ? ◀─ actif ──▶ proposition journalisée, NON exécutée
   │ non
   ▼
Journalise la proposition
   │
   ▼
RequiresConfirmation ? ── oui ──▶ Confirmez-vous ? [o/n]
   │  non                              │ non
   ▼                                   ▼
Exécution réelle          refus → journalisé + rejeté
   │
   ▼
Journalise le résultat d'exécution
```

**Garantie clé** : le LLM peut proposer ce qu'il veut ; seules les actions **explicites**
de la whitelist sont jamais exécutées. Le modèle ne peut pas inventer une action
autorisée.

---

## Modèle IA local (Ollama)

- **Ollama** : serveur HTTP local, `http://127.0.0.1:11434` (surchargeable par `OLLAMA_HOST`).
- **Modèle par défaut** : `phi3:mini` (`AMANE_LLM_MODEL` pour changer).
- Endpoint utilisé : `POST /api/generate` avec `stream: false` (réponse JSON complète).
- **Aucun appel réseau sortant** : le modèle est appelé en local uniquement.
- Les prompts ne contiennent **jamais** de clé (AK/DEK/KEK) ni de payload chiffré —
  seulement de l'état du cluster (nœuds, séquences, types d'opération).

---

## Sécurité & règles non négociables

1. **Jamais de clé en clair** : les agents (Sentinelle/Copilote) ne transmettent au
   modèle que des métadonnées (nœuds, séquences, types d'opération). Pas de AK/DEK/KEK,
   pas de payload chiffré en clair dans les prompts ni dans les logs.
2. **Liste blanche stricte** : aucune action non explicitement whitelistée ne peut être
   exécutée. La section `NOTE` de `whitelist.go` rappelle que les actions lourdes
   (rotation de clé, redémarrage machine, récupération Shamir) sont **exclues par
   principe** (escalade humaine seule).
3. **Confirmation humaine obligatoire** pour les actions marquées `RequiresConfirmation`.
4. **Dry-run** : en mode test, le Copilote propose mais n'exécute jamais.
5. **Interface read-only** : `ClusterReader` est la seule porte d'entrée vers le cluster ;
   jamais d'accès direct etcd/PostgreSQL depuis le module IA.

---

## Jalons de validation (M1 → M4)

| Jalon | Objectif | Critère |
|---|---|---|
| **M1** | Sentinelle en mode texte seul | ≥ 80 % des questions fréquentes (fixtures) avec couverture de mots-clés suffisante |
| **M2** | Liste blanche stricte | **100 %** des actions non whitelistées rejetées, y compris tentatives adverses (`rm -rf /`, `rotate_key`, `disable_fencing`, `exfiltrate_data`, …) |
| **M3** | Mode dry-run + simulateur de pannes | Diagnostic cohérent avec la panne injectée ; **aucune** action exécutée en dry-run ; aucune action critique proposée |
| **M4** | Confirmation utilisateur réelle + journalisation | L'acceptation **ET le refus** sont journalisés (2 événements distincts : confirmation ≠ exécution) |

Les tests M1–M4 sont **100 % déterministes** : ils utilisent un **stub LLM**
(`stub_llm_test.go`) injecté via `SetLLM`, et un **simulateur de pannes**
(`simulator/failure.go`). **Aucun Ollama ni réseau requis** pour les tests.

---

## Installation & mise en route

Prérequis :
- **Go ≥ 1.26**
- **Ollama** installé et démarré (pour branche sur un vrai modèle) :
  ```bash
  ollama serve
  ollama pull phi3:mini
  ```
- Code de Mission C à la racine (pour satisfaire le `replace` du `go.mod`).

```bash
# 1. Depuis la racine du monorepo (ai-assistant dépend de orchestrator-go/clusterreader)
go build ./ai-assistant/...

# 2. Tests
cd ai-assistant && go test ./... -count=1 -race

# 3. Mise en route manuelle (Ollama)
ollama serve
ollama pull phi3:mini
```

> En environnement de test (CI), aucun Ollama n'est requis : les tests utilisent le
> stub LLM injectable.

---

## Tests

```bash
cd ai-assistant && go test ./... -count=1 -race
```

| Suite | Couvre |
|---|---|
| `m1_test.go` | Sentinelle mode texte (M1) |
| `m2_test.go` | Liste blanche + tentatives adverses (M2) |
| `m3_test.go` | Dry-run + scénarios de panne (M3) |
| `m4_test.go` | Confirmation + journalisation acceptation/refus (M4) |
| `llm/llm_test.go` | Client Ollama : OK, erreur HTTP, réponse vide, env (httptest) |
| `stub_llm_test.go` | Stub déterministe (injection `SetLLM`) |

---

## Variables d'environnement

| Variable | Défaut | Rôle |
|---|---|---|
| `OLLAMA_HOST` | `http://127.0.0.1:11434` | Adresse HTTP locale d'Ollama |
| `AMANE_LLM_MODEL` | `phi3:mini` | Modèle local utilisé par les agents |