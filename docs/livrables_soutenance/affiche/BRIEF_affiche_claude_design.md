# Brief — Affiche de soutenance à reproduire (pour Claude Design)

> **À Claude** : génère une **affiche de soutenance A1 portrait (594 × 841 mm), une seule page**, en HTML/CSS autonome (imprimable en PDF via `@page { size: 594mm 841mm; margin:0 }`). Ambiance **sombre premium**, charte **EL BARAA CONSULT**. Toutes les images existent déjà (chemins ci-dessous). Le texte doit être net (vrai texte HTML, pas d'image). Tout doit tenir sur **une seule page** sans débordement.

---

## 1. Format & ambiance

- **Format** : A1 portrait (594 × 841 mm), une page, marges 0.
- **Ambiance** : dark luxury — fond **bordeaux profond → noir** avec halos rouge/or, motif géométrique zellige **très discret** en filigrane (faible opacité, masqué au centre pour la lisibilité).
- **Direction** : sobre, structuré, premium ; sections nettement séparées par des intertitres dorés en capitales.

## 2. Charte couleur (EL BARAA CONSULT) — à respecter strictement

| Rôle | HEX |
|---|---|
| Rouge principal | `#B3121B` |
| Rouge foncé | `#8E0E15` |
| Fond bordeaux profond | `#1A0306` → `#2B0508` |
| Doré principal | `#C79A1B` |
| Doré clair | `#D8AE35` |
| Texte ivoire | `#F8F1E7` |
| Texte atténué (beige-rosé) | `#D8B4A6` |

Accents en doré, points forts en rouge. **Pas de bleu, pas de vert** (sauf la pastille « sain » verte des niveaux de risque).

## 3. Images disponibles (déjà générées — à intégrer)

| Fichier | Rôle sur l'affiche |
|---|---|
| `logoeisgie.png` | Logo **EIGSI Casablanca** (école) — bandeau haut gauche, dans une pastille blanche |
| `logoentreprise.png` | Logo **EL BARAA CONSULT** (entreprise) — bandeau haut droite, dans une pastille blanche |
| `images/hero.png` | Visuel hero : bouclier doré protégeant un cluster de serveurs rouges + capsules chiffrées — à côté du titre |
| `images/cle_dek.png` | Icône crypto (clé + bouclier + enveloppe scellée) — section « clé DEK » |
| `images/stock_crop.png` | Capture réelle : tableau de bord stock du logiciel métier (preuve) |
| `images/repl_crop.png` | Capture réelle : terminal montrant la réplication des comptes sur le standby (preuve) |
| `images/afrique.png` | (optionnel) Bouclier protégeant l'Afrique — accent identitaire si la place le permet |

> Les captures doivent être **entièrement visibles** (pas rognées par CSS) et **lisibles** ; fond blanc derrière, `object-fit: contain`.

## 4. Structure de l'affiche (de haut en bas)

1. **Bandeau logos** : EIGSI (gauche) · EL BARAA (droite), pastilles blanches.
2. **En-tête** : surtitre + titre + hero.
3. **Le problème** (encadré citation).
4. **L'architecture** — 3 acteurs + frontière chiffrée.
5. **La clé DEK** — icône + 3 étapes.
6. **Preuves** — 2 captures réelles.
7. **Supervision intelligente** — agent IA Mistral.
8. **Validé sur banc** (liste) + **Résultats** (métriques) + **carte auteur**.

---

## 5. CONTENU EXACT (texte à afficher)

### Surtitre (eyebrow)
`SOUVERAINETÉ DES DONNÉES · ZERO-KNOWLEDGE · STAGE DE FIN D'ÉTUDES 2026`

### Titre
- En grand, en arabe doré : **أمان** (police arabe type *Amiri* / *Noto Naskh Arabic*)
- À côté : **Amān** — sous-titre : *Framework SaaS Souverain*

### Accroche (tagline)
> La donnée métier reste sur les machines du client. L'éditeur gère comptes et licences — et reste **cryptographiquement incapable** de lire les données de ses clients.

### Le problème (encadré italique, barre dorée à gauche)
> Comment un éditeur peut-il vendre et maintenir un logiciel métier en mode SaaS (comptes, licences, mises à jour) sans jamais accéder aux données de ses clients — conformément au cadre africain de protection des données (AUDPF) ?

### L'architecture — trois acteurs, une frontière chiffrée

**Zone « Chez l'éditeur — ne voit jamais les données métier » :**
- **SaaS éditeur** — Comptes tenants, licences, suivi du parc. Django + PostgreSQL. — étiquette : *voit : compte / licence*
- **Relais zero-knowledge** — Stocke des blobs chiffrés opaques + code de récupération chiffré. Même saisi, il ne déchiffre rien. — étiquette : *voit : jamais le clair*

**Bande centrale (frontière)** : `🔐 Frontière cryptographique — tout sort chiffré (DEK · libsodium)`

**Zone « Chez la PME — périmètre souverain (données en clair) » :**
- **Nœud primaire** — Exécute le logiciel, détient et chiffre les données. PostgreSQL primaire (écritures). — étiquette : *PRIMAIRE · actif*
- **Nœud standby** — Réplique en continu. Lecture seule. Prend le relais par quorum si le primaire tombe. — étiquette : *STANDBY · réplica*

**Ligne sous la zone PME** : `⟲ Réplication streaming PostgreSQL < 1 s · fencing par jeton d'époque (anti split-brain)`

### La clé DEK — une seule clé par entreprise, jamais vue par l'éditeur
(icône `images/cle_dek.png` + 3 étapes côte à côte avec flèches)
1. **🔑 DEK** — Clé symétrique unique par entreprise. Chiffre **les données métier + le journal CBOR** avant tout écrit disque.
2. **📲 Par appareil** — La DEK est emballée en **sealed box (X25519)** pour chaque appareil autorisé. *Enrôlement par QR code : perspective.*
3. **🆘 Récupération** — Emballée sous un **code haute entropie (Argon2id)** → blob chiffré stocké sur le relais. L'éditeur ne déchiffre rien.

### Preuves — le logiciel tourne (banc multi-OS réel)
- Capture `images/stock_crop.png` — légende : **Logiciel métier en service** — tableau de bord stock (nœud PME, données en clair, alertes de seuil).
- Capture `images/repl_crop.png` — légende : **Réplication vérifiée** — les comptes créés sur le primaire apparaissent sur le standby (Ubuntu).

### Supervision intelligente — agent IA (Mistral) qui ne voit jamais les données
(badge agent 🧠 + 3 volets avec flèches)
1. **📊 Analyse** — Rôles PostgreSQL, réplication streaming, failover, régularité des heartbeats — **métriques d'infrastructure uniquement**.
2. **🩺 Verdict** — Score d'anomalie **0–100** + diagnostic et action recommandée en langage naturel. (afficher 3 pastilles de niveau : `sain` vert · `surveiller` doré · `critique` rouge)
3. **🛡️ Garanties** — **Zero-knowledge** : aucune donnée métier transmise. **Fail-safe** : repli local déterministe si Mistral indisponible.

### Validé sur banc multi-OS (liste à coches dorées ✓)
- Chiffrement **DEK cross-OS** (Windows · Ubuntu · Kali · Debian) — libsodium.
- **Journal append-only CBOR** chiffré (DEK) avant écriture disque.
- Réplication PostgreSQL primaire↔standby + **failover par quorum & fencing**.
- Récupération **zero-knowledge** (code haute entropie · Argon2id · blob sur relais).
- **Dé-enrôlement** d'appareil + 2 tenants souverains indépendants.
- → *Perspective : enrôlement d'appareil par QR code (primitive sealed box X25519 prête).* (en gris, flèche → au lieu de ✓)

### Résultats (3 métriques en gros chiffres dorés)
- **6/6** — scénarios métier
- **4/4** — protections de résilience
- **2** — tenants souverains

### Carte auteur (en bas, encadré dégradé rouge→or)
- **MPIGA-ODOUMBA Jesse**
- EIGSI Casablanca — Spécialité Big Data & Intelligence Artificielle · Promotion 2026
- Stage de fin d'études · AL BARAA CONSULTING, Casablanca · **Soutenance 2026**

---

## 6. Règles d'exactitude (NE PAS survendre au jury)

- L'**enrôlement par QR code n'est PAS implémenté** → toujours le présenter comme **perspective** (seule la primitive sealed box X25519 est faite).
- L'**agent IA Mistral est réel** (supervision d'infrastructure, zero-knowledge, repli local) → à valoriser.
- Ne pas réintroduire de section « Pile technique » (retirée volontairement) ; les technos apparaissent déjà dans les sections.

## 7. Sortie attendue

- Un fichier **HTML autonome** (CSS inline), dimensions A1, **une seule page**, prêt à exporter en PDF (Chrome → Imprimer → PDF → A1, marges aucune, graphiques d'arrière-plan cochés).
- Texte impeccable (français + arabe), images intégrées via les chemins ci-dessus, charte couleur respectée.
