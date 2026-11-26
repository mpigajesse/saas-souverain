### A-07 — Rejeu de génération de la pile (réplicabilité)

- Gabarit : `templates/tenants/docker-compose.yml.tmpl`
- Contexte (même tenant) : `tools/bench/pile-bench.json` — tenant « AMANE Test » (ea49c4c4-f46f-464d-b5db-4fb5928b7e49)

#### 1. Reproductibilité (même secret, deux horodatages)
- Lignes de diff : 10
```
--- reproductibilite/A
+++ reproductibilite/B
@@ -1,6 +1,6 @@
 # -----------------------------------------------------------------------------
 # AMANE — Pile PME, machine SERVEUR — AMANE Test
-# Générée le : 25/11/2026 à 21:00
+# Générée le : 25/11/2026 à 21:05
 #
 # DÉFINITION UNIQUE DE LA PILE (refonte V3, ADR-001, ADR-004, ADR-005). Les
 # installeurs Windows et Linux la téléchargent depuis le portail ; Windows et
```

#### 2. Secrets (même horodatage, deux jetons de relais)
- Lignes de diff : 11
```
--- secrets/A
+++ secrets/B
@@ -197,7 +197,7 @@
       # Jeton du relais propre au tenant (ADR-011) : il ouvre ses seuls blobs
       # et sa seule topologie. Fixé par le portail : retélécharger la pile
       # suffit à le renouveler.
-      RELAY_AUTH_TOKEN: "bfe31518d11a37ae096cbe6b3890e8ae6f6a899d6fe14e183cd0a2ea4d6794b6"
+      RELAY_AUTH_TOKEN: "0000000000000000000000000000000000000000000000000000000000000000"
       SAAS_URL: "${SAAS_URL}"
       NODE_ADDR: "${NODE_IP}:${WEB_PORT:-9001}"
       # active = premier serveur : génère la DEK au premier démarrage.
```
