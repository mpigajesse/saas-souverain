# Profil LinkedIn — Fiche d'expérience (PFE)

> Jesse MPIGA-ODOUMBA — Filière Expert (FE), Promo 2026 · EIGSI
> ⚠️ Ceci est la **description d'expérience du profil**, pas une publication.
> Pour le post de soutenance, voir `MPIGA-ODOUMBA_Jesse_FE_Promo2026_Post_LinkedIn.md`.

---

## Champs du formulaire LinkedIn

| Champ | Valeur |
| --- | --- |
| **Intitulé du poste** | Stagiaire Fin d'Études – Architecture Data & IA |
| **Type d'emploi** | Stage |
| **Organisation** | AL BARAA CONSULTING - SARL |
| **Lieu** | Casablanca, Maroc |
| **Type de lieu** | Sur site *(ou Hybride selon ta réalité)* |
| **Période** | *(mois de début)* 2026 – juillet 2026 |

---

## Description — version recommandée

> À copier-coller tel quel dans le champ « Description ».
> ~1 600 caractères (limite LinkedIn : 2 000).

```text
Conception et implémentation d'une architecture Coffre-Fort Data P2P Souveraine : une plateforme où les données métier restent sur les machines du client et ne transitent que chiffrées. L'éditeur gère comptes et licences sans jamais pouvoir lire les données (zero-knowledge).

▸ ARCHITECTURE & PROTOCOLE P2P
• Conception de l'architecture complète à trois acteurs : SaaS éditeur, relais zero-knowledge, cluster souverain chez le client
• Implémentation du protocole P2P avec chiffrement de bout en bout — cœur cryptographique en Rust / libsodium (XChaCha20-Poly1305, X25519, Argon2id)
• Haute disponibilité : réplication PostgreSQL primaire/standby, bascule automatique par quorum, fencing par jeton d'époque
• Enrôlement d'appareil par QR code et procédure de récupération sans jamais exposer les clés

▸ DATA & IA
• Intégration d'agents IA distribués pour la gestion et la supervision des données, exécutés dans le périmètre souverain du client
• Journal d'événements append-only chiffré (CBOR) servant de socle auditable

▸ CADRAGE & CONFORMITÉ
• Rédaction du Plan Directeur et de la documentation technique complète
• Validation de la conformité aux exigences de souveraineté numérique
• Validation du socle technique sur banc de 3 machines réelles (Windows 11 / Ubuntu / Kali) avant tout développement métier

Environnement technique : Rust, libsodium, PostgreSQL, Docker multi-architecture, Django, React/TypeScript, CBOR, stockage objet S3/MinIO.
```

---

## Description — version courte

> Si tu préfères une fiche compacte (~600 caractères).

```text
Conception et implémentation d'une architecture Coffre-Fort Data P2P Souveraine : les données métier restent chez le client et ne sortent que chiffrées, l'éditeur ne pouvant jamais les lire (zero-knowledge).

• Conception de l'architecture souveraine à trois acteurs
• Implémentation du protocole P2P avec chiffrement de bout en bout (Rust / libsodium)
• Haute disponibilité : réplication PostgreSQL, failover par quorum, fencing
• Intégration d'agents IA distribués pour la gestion des données
• Rédaction du Plan Directeur et de la documentation technique complète
• Validation de la conformité aux exigences de souveraineté numérique

Environnement : Rust, libsodium, PostgreSQL, Docker, Django, React/TypeScript.
```

---

## Compétences à associer à cette expérience

LinkedIn permet de rattacher jusqu'à **5 compétences** par expérience — elles pèsent
beaucoup dans le matching recruteur. Choisis-les dans cet ordre de priorité :

1. **Architecture logicielle** *(ou Architecture de données)*
2. **Cryptographie**
3. **Rust**
4. **Intelligence artificielle** *(ou Systèmes distribués)*
5. **PostgreSQL**

À ajouter dans la section Compétences du profil (hors des 5 rattachées) :
Systèmes distribués · Sécurité informatique · Peer-to-peer (P2P) · Docker ·
Django · TypeScript · Haute disponibilité · Souveraineté numérique · CBOR.

---

## Titre de profil (headline) — suggestions

Le titre est ce qui apparaît partout : recherches, commentaires, invitations.
Il compte plus que la fiche d'expérience elle-même.

```text
Élève-ingénieur EIGSI (Promo 2026) · Architecture Data & IA — Systèmes distribués souverains, cryptographie appliquée, Rust
```

```text
Ingénieur Data & IA en devenir · EIGSI 2026 — Architectures souveraines, P2P chiffré, IA distribuée
```

---

## Points d'attention

- **Pas de markdown** : LinkedIn n'interprète ni `**gras**` ni `#`. Les blocs
  ci-dessus n'utilisent que des puces `•` et des flèches `▸`, qui passent en
  Unicode tel quel. Ne rajoute pas d'astérisques.
- **Les 2 premières lignes seulement sont visibles** avant le « …voir plus ».
  La phrase d'ouverture doit donc porter à elle seule tout le projet — c'est
  pourquoi elle contient déjà « Coffre-Fort Data P2P Souveraine » et
  « zero-knowledge ».
- **Mots-clés = visibilité** : les recruteurs filtrent sur des termes exacts
  (Rust, PostgreSQL, cryptographie, systèmes distribués, IA). Ils sont
  volontairement écrits en toutes lettres, jamais sous-entendus.
- **Cohérence avec le post de soutenance** : le post reprend le même vocabulaire
  (« Coffre-Fort Data P2P Souveraine », « Architecture Data & IA »). Un visiteur
  qui passe du post au profil retrouve le même projet — ne reformule pas l'un
  sans l'autre.
- **Ajoute un média** : LinkedIn permet de joindre un document à l'expérience.
  Un PDF de 3–4 slides (schéma d'architecture + résultats du banc de test) vaut
  bien plus que du texte supplémentaire.
