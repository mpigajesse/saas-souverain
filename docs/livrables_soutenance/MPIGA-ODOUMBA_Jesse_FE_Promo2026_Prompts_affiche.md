# Prompts — Images pour l'affiche de soutenance (GPT Image 2 → assemblage Claude/Canva)

> **Projet** : Framework SaaS Souverain — **Amān (أمان)** · souveraineté des données.
> **Auteur** : MPIGA Jesse · EIGSI Casablanca (Big Data & IA) · Promo 2026 · Stage AL BARAA CONSULTING.
> **But** : générer de **belles images** dans GPT Image 2, puis les **assembler** (avec les logos + le texte) dans une affiche pro.

---

## 🎨 Charte couleur à respecter (EL BARAA CONSULT)

> Toutes les images doivent utiliser **cette palette** pour rester cohérentes avec la marque et les logos.

| Rôle | Couleur | HEX |
|---|---|---|
| Rouge principal (branding, dominante) | 🔴 | `#B3121B` |
| Rouge foncé (profondeur, ombres) | 🟥 | `#8E0E15` |
| Doré principal (accents premium) | 🟡 | `#C79A1B` |
| Doré clair (lumière, reflets) | 🟨 | `#D8AE35` |
| Blanc / ivoire (fond propre, respiration) | ⬜ | `#FFFFFF` |

**Formulation à coller dans chaque prompt** (déjà incluse plus bas) :
> *« strict color palette: deep crimson red #B3121B, dark red #8E0E15, rich gold #C79A1B, light gold #D8AE35, and white/ivory. No blue, no green. »*

---

## ⚠️ À lire avant de générer

1. **Pas de texte dans les images générées.** Les IA déforment les mots (surtout français/arabe). On génère des **visuels sans texte**, et on ajoute le titre/texte à l'assemblage. (Tous les prompts finissent par `no text, no letters, no words`.)
2. **Pas de logos générés par l'IA.** Tes vrais logos (EIGSI + EL BARAA) seront **posés à l'assemblage** (voir section Logos). Ne demande jamais à l'IA de dessiner un logo.
3. **Format** : précise le ratio. Pour des éléments d'affiche → carré `1024x1024` (illustrations à intégrer) ou portrait `1024x1536` (si fond plein cadre).
4. **Génère 3-4 variantes** par prompt et garde la meilleure.
5. **Deux ambiances possibles** selon ton goût :
   - **Fond clair** (blanc/ivoire + rouge + or) → officiel, propre, recommandé par la charte.
   - **Fond sombre** (bordeaux profond + or) → premium, dramatique. Choisis une seule ambiance pour toute l'affiche.

---

## 🖼️ Stratégie : 4 images à générer, puis assembler

Plutôt que 5 affiches concurrentes, on génère **4 images réutilisables** qui s'assemblent en UNE affiche pro :

| Image | Rôle dans l'affiche | Prompt |
|---|---|---|
| **A — Hero/visuel principal** | grande illustration en haut/centre | Prompt 1 |
| **B — Schéma 3 acteurs** | bande architecture au milieu | Prompt 2 |
| **C — Motif/texture de fond** | arrière-plan discret (zellige) | Prompt 3 |
| **D — Icône crypto (clé/bouclier)** | petit accent décoratif | Prompt 4 |

---

## 🚀 VERSION PRÊTE À COLLER (copier-coller direct dans GPT Image 2)

### ✅ Image A — Hero (bouclier + cluster souverain)

```
Génère une image carrée haute qualité, sans aucun texte. Description : A premium illustration symbolizing data sovereignty and zero-knowledge encryption. A glowing protective shield made of light wrapping around a stylized cluster of connected server nodes; encrypted data leaves the cluster as sealed luminous capsules. Elegant, sophisticated, editorial tech style, soft volumetric lighting, subtle North-African / Moroccan geometric (zellige) motifs in the background at low opacity. Strict color palette: deep crimson red #B3121B, dark red #8E0E15, rich gold #C79A1B, light gold #D8AE35, on a clean white / ivory background. No blue, no green. High detail, refined. No text, no letters, no words.
```

### ✅ Image B — Schéma des 3 acteurs (infographie)

```
Génère une image au format portrait vertical, ratio 2:3, haute qualité, sans aucun texte. Description : A clean modern infographic showing THREE distinct zones connected by encrypted data flows. ZONE 1 (top): a cloud server building icon, a SaaS editor handling only accounts and licenses. ZONE 2 (middle): a relay server as an opaque locked vault holding sealed encrypted blobs it can never open (zero-knowledge). ZONE 3 (bottom): a small business office with two or three connected computers forming a local cluster where the real data lives. Golden encrypted capsules travel along the connections. Refined vector style, soft shadows, generous spacing, empty label areas near each zone for captions. Strict color palette: deep crimson red #B3121B, dark red #8E0E15, rich gold #C79A1B, light gold #D8AE35, on white / ivory. No blue, no green. No text, no letters, no words.
```

### ✅ Image C — Texture de fond (zellige discret)

```
Génère une image carrée haute qualité, sans aucun texte. Description : An elegant subtle background texture inspired by Moroccan zellige and Sahelian geometric patterns, very low contrast, refined and premium, suitable as a poster background behind text. Delicate interlocking geometric shapes. Strict color palette: warm ivory and white base with faint deep crimson red #B3121B and rich gold #C79A1B line work, very soft, understated. No blue, no green. Seamless, flat, minimal. No text, no letters, no words.
```

### ✅ Image D — Icône crypto (clé / coffre zero-knowledge)

```
Génère une image carrée haute qualité, fond transparent ou blanc, sans aucun texte. Description : A single elegant symbolic icon of cryptographic protection: a glowing master key merged with a shield, radiating light, with a small sealed encrypted envelope. Premium, polished, 3D soft render. Strict color palette: deep crimson red #B3121B and rich gold #C79A1B with light gold #D8AE35 highlights, on white. No blue, no green. Centered, clean, icon style. No text, no letters, no words.
```

### ✅ Image F — Hiérarchie de clés DEK (cœur cryptographique)

```
Génère une image carrée haute qualité, fond blanc ou transparent, sans aucun texte. Description : An elegant symbolic illustration of a cryptographic key hierarchy. A single large glowing master key at the center (the DEK, one per company) radiating light; from it, branches lead to: business data files and an append-only journal being wrapped into sealed luminous envelopes; several smaller keys, one per authorized device; and a separate locked safe holding an encrypted recovery blob. Premium polished 3D soft render, refined. Strict color palette: deep crimson red #B3121B, dark red #8E0E15, rich gold #C79A1B, light gold #D8AE35, on white. No blue, no green. Centered, balanced composition. No text, no letters, no words.
```

> Sur l'affiche, la clé DEK est illustrée par l'icône **`images/cle_dek.png`** + un encadré « DEK → par appareil (sealed box X25519) → récupération (Argon2id) ». Tu peux remplacer cette icône par l'image F si tu préfères une illustration plus complète de la hiérarchie de clés.

### ✅ (Bonus) Image E — Afrique / souveraineté

```
Génère une image au format portrait vertical, ratio 2:3, haute qualité, sans aucun texte. Description : An inspiring visual about digital data sovereignty for Africa. A luminous stylized African continent made of interconnected glowing network nodes, protected inside a translucent shield of light, golden encrypted capsules orbiting it, subtle Moroccan/Sahelian geometric motifs framing the edges. A sense of trust, protection and emancipation. Strict color palette: deep crimson red #B3121B, dark red #8E0E15, rich gold #C79A1B, light gold #D8AE35, on white / ivory. No blue, no green. Cinematic soft lighting, premium editorial style, empty space top and bottom. No text, no letters, no words.
```

---

## 🏷️ Logos — à NE PAS générer, à poser à l'assemblage

Tu as déjà les vrais logos dans le dépôt :

| Logo | Fichier |
|---|---|
| **EIGSI Casablanca** (école) | `docs/livrables_soutenance/affiche/logoeisgie.png` |
| **EL BARAA CONSULT** (entreprise) | `frontend/public/logoentreprise.png` |

**Placement recommandé sur l'affiche** :
- En **haut** : les deux logos côte à côte (école à gauche, entreprise à droite) OU en bandeau partenaires.
- En **pied de page** : reprise discrète + ton nom et la mention soutenance.
- Garde-les sur un **fond clair** ou dans une **pastille blanche** pour qu'ils restent lisibles.

> ⚠️ Vérifie que `logoentreprise.png` a un **fond transparent** ; sinon, place-le dans un cartouche blanc arrondi.

---

## 🧩 Comment assembler l'affiche pro (avec Claude / Canva)

1. **Génère** les images A, B, C (+ D, E si tu veux) dans GPT Image 2, télécharge-les.
2. **Dépose-les** dans `docs/livrables_soutenance/affiche/` (ex. `hero.png`, `schema.png`, `fond.png`, `icone.png`).
3. **Assemblage** — deux voies :
   - **Voie Claude (HTML)** : demande-moi *« intègre ces images + les 2 logos dans l'affiche HTML avec la charte rouge/or »*. Je place les images, les logos et le texte dans `affiche_soutenance.html` → tu exportes en PDF A1. (Le texte reste parfait car il est en vrai HTML.)
   - **Voie Canva** : importe les images + logos, pose le bloc de texte ci-dessous, exporte en PDF haute résolution.

---

## ✍️ Bloc de texte à poser sur l'affiche (copier-coller)

**TITRE**
> أمان · Amān — Framework SaaS Souverain

**ACCROCHE**
> La donnée métier reste sur les machines du client. L'éditeur gère comptes et licences — sans jamais lire les données.

**LE PROBLÈME**
> Comment un éditeur peut-il vendre et maintenir un logiciel métier en mode SaaS sans jamais accéder aux données de ses clients ? (cadre africain AUDPF)

**LA RÉPONSE — 3 acteurs**
> 1. **SaaS éditeur** — comptes & licences uniquement.
> 2. **Relais zero-knowledge** — blobs chiffrés, ne voit jamais le clair.
> 3. **Cluster PME** — exécute le logiciel, détient et chiffre ses données.

**PREUVES (spike Phase 0)**
> - DEK cross-OS (Windows · Ubuntu · Kali · Debian) — libsodium.
> - Réplication PostgreSQL primaire ↔ standby (< 1 s) + quorum.
> - Enrôlement par QR code (sealed box X25519).
> - Récupération zero-knowledge (Argon2id).
> - Agent IA (Mistral) supervise sans voir les données.
> - **6/6 scénarios métier · 4/4 protections de résilience · 2 tenants souverains.**

**PILE TECHNIQUE**
> Rust · libsodium (XChaCha20-Poly1305, X25519, Argon2id) · PostgreSQL · Docker · CBOR · Django.

**PIED DE PAGE**
> MPIGA Jesse · EIGSI Casablanca — Spécialité Big Data & IA · Promotion 2026
> Stage de fin d'études — AL BARAA CONSULTING, Casablanca · Soutenance 2026

---

## 💡 Astuce orthographe arabe

Le mot **أمان** (« Amān ») est souvent mal rendu par les IA d'image.
→ **Ne le fais pas générer.** Tape-le à l'assemblage avec une vraie police arabe (*Amiri*, *Noto Naskh Arabic*), en doré `#C79A1B` ou rouge `#B3121B`.
