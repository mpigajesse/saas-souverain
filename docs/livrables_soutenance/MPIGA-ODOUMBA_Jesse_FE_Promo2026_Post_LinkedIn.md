# Post LinkedIn — Soutenance Projet de Fin d'Études

> Jesse MPIGA-ODOUMBA — Filière Expert (FE), Promo 2026 · EIGSI
> Soutenance : **10 juillet 2026, 10h00**
> Poste : **Stagiaire Fin d'Études – Architecture Data & IA** · AL BARAA CONSULTING (Casablanca)
> Projet : **Coffre-Fort Data P2P Souveraine** (nom de code interne : *SaaS Souverain*)

---

## Version principale (impactante — recommandée)

**Et si votre éditeur de logiciel ne pouvait PAS lire vos données ? Même sous la contrainte. Même s'il le voulait.**

Le 10 juillet, j'ai soutenu mon Projet de Fin d'Études : **la conception et l'implémentation d'une architecture Coffre-Fort Data P2P Souveraine.** Et j'ai défendu une conviction.

Aujourd'hui, faire confiance à un SaaS, c'est lui livrer vos données les plus sensibles — stock, facturation, paie — sur *ses* serveurs, sous *sa* garde. On appelle ça « la confiance ». Moi, j'appelle ça un angle mort.

Alors, comme Stagiaire Fin d'Études – Architecture Data & IA chez AL BARAA CONSULTING, j'ai construit l'inverse.

Le principe repose sur une promesse que la cryptographie rend *vérifiable*, pas seulement *promise* : **zero-knowledge.**

- Vos données restent chez vous, sur vos machines, en **pair-à-pair**.
- Elles ne sortent que **chiffrées de bout en bout**.
- L'éditeur gère vos comptes et vos licences — **jamais vos clés, jamais vos données.**

Même si son serveur est piraté. Même s'il est saisi. Il ne peut rien lire. Ce n'est pas une politique de confidentialité : c'est une **impossibilité mathématique.**

Trois acteurs, une seule vérité :
- un **SaaS éditeur** qui ne voit que comptes et licences ;
- un **relais zero-knowledge** qui ne stocke que des blocs chiffrés, aveugles ;
- un **cluster pair-à-pair chez le client**, seul détenteur des clés.

Ce que j'ai porté de bout en bout, et qui m'a fait grandir en tant qu'ingénieur :

**Architecture & protocole**
- la conception complète de l'architecture Coffre-Fort Data P2P Souveraine ;
- l'implémentation du protocole P2P avec **chiffrement de bout en bout** — cœur cryptographique en **Rust** (libsodium), zéro primitive réinventée à la main ;
- de la **haute disponibilité** : réplication PostgreSQL, bascule automatique par quorum, *fencing* par jeton d'époque ;
- un **enrôlement par QR code** et une **récupération** possible même si le client perd *toutes* ses machines — sans jamais briser le secret.

**Data & IA**
- l'intégration d'**agents IA distribués** pour la gestion et la supervision des données — l'intelligence s'exécute **là où vivent les données**, dans le périmètre souverain du client, jamais sur un serveur tiers ;
- un **journal chiffré, append-only**, qui trace tout sans rien exposer — et sert de socle auditable aux agents.

**Cadrage & conformité**
- la rédaction du **Plan Directeur** et de la documentation technique complète ;
- la **validation de la conformité aux exigences de souveraineté numérique.**

Ma plus grande leçon ? La plus dure n'a pas été le code. **Ça a été la discipline** : prouver le socle — crypto, journal, failover — sur 3 machines réelles (Windows, Ubuntu, Kali) **avant** d'écrire la moindre ligne de métier. Parce qu'un socle qui cède, c'est tout le reste à réécrire.

Et une conviction qui s'est renforcée en chemin : **l'IA souveraine n'est pas une IA plus faible.** C'est une IA qui accepte de venir jusqu'à la donnée, au lieu d'exiger que la donnée vienne à elle.

Merci à @Soumia CHOKRI, Directrice Générale d'@AL BARAA CONSULTING et mon encadrante entreprise, pour la confiance accordée en me confiant la responsabilité complète de ce projet stratégique. Merci à @Ayoub AMRANI, mon tuteur à l'EIGSI, pour son suivi et son exigence méthodologique. Et merci au jury pour ses questions qui piquent.

Merci, enfin, à l'ensemble des enseignants de l'@EIGSI — campus de Casablanca comme de La Rochelle — pour la qualité et la rigueur de la formation. Un merci tout particulier à mes deux professeurs clés de la dominante IA & Big Data, @Sohaib Baroud et @Badr-eddine BEN EL MOSTAFA, dont l'exigence a façonné ma manière de penser la donnée.

La souveraineté numérique n'est pas un argument marketing. **C'est une décision d'architecture.** Et je crois que c'est l'une des grandes directions du logiciel — et de l'IA — de demain.

Le diplôme est une ligne d'arrivée. Le projet, lui, ne fait que commencer.

*Vous construisez du logiciel métier ou déployez de l'IA sur données sensibles, et cette approche vous parle ? Écrivez-moi — j'en parle avec plaisir.*

```text
#PFE #Soutenance #EIGSI #SouveraineteNumerique #Cybersécurité #ZeroKnowledge #P2P #Rust #IA #DataArchitecture #Promo2026 #AlBaraaConsulting
```

---

## Version courte (accroche brute — stories / repost)

**Mon éditeur de logiciel ne peut pas lire mes données. Par conception.**

Le 10 juillet 2026, j'ai soutenu mon Projet de Fin d'Études, réalisé chez @AL BARAA CONSULTING comme **Stagiaire Fin d'Études – Architecture Data & IA** : la conception et l'implémentation d'une **architecture Coffre-Fort Data P2P Souveraine.**

Le principe ? *Zero-knowledge.* Vos données restent chez vous, en pair-à-pair, ne sortent que chiffrées de bout en bout, et l'éditeur ne détient **jamais** les clés. Piraté ou saisi, son serveur ne révèle rien.

Pas une promesse. Une impossibilité mathématique.

Protocole P2P chiffré · cœur crypto en Rust · failover PostgreSQL par quorum · **agents IA distribués s'exécutant là où vivent les données** · Plan Directeur et documentation technique complète.

Merci à @Soumia CHOKRI (@AL BARAA CONSULTING), à @Ayoub AMRANI et à tous les enseignants de l'@EIGSI (Casablanca & La Rochelle) — mention spéciale à @Sohaib Baroud et @Badr-eddine BEN EL MOSTAFA, mes profs de la dominante IA & Big Data.

Le diplôme est une ligne d'arrivée. Le projet ne fait que commencer.

```text
#PFE #SouveraineteNumerique #ZeroKnowledge #P2P #Cybersécurité #Rust #IA #EIGSI #Promo2026
```

---

## Conseils de publication

- **Timing** : même passée, la soutenance reste 100 % postable — le format
  « je viens de vivre une étape » fonctionne jusqu'à ~2 semaines après. Publie
  un mardi–jeudi, 8h–10h ou 17h–18h.
- **La 1ère ligne fait tout** : elle doit intriguer *seule*, avant le « …voir plus ».
  Ici, la question provocante en ouverture est l'hameçon — ne la coupe pas.
- **Ajoute une photo** : toi devant tes slides / le jury. Les posts avec visage
  captent nettement plus de portée qu'un mur de texte.
- **Hashtags** : 5 à 8 max. Copie-les depuis le bloc `text` (pas de titres parasites).
- **Mentions `@`** : les `@Nom` du texte ne sont PAS cliquables au copier-coller.
  Dans l'éditeur LinkedIn, retape le `@` puis le nom et **sélectionne le vrai
  profil / la vraie page** dans la liste déroulante (AL BARAA CONSULTING, EIGSI,
  Soumia CHOKRI, Ayoub AMRANI, Sohaib Baroud, Badr-eddine BEN EL MOSTAFA). Sans
  cette sélection, la personne n'est pas notifiée.
- **Lien** : garde le corps du post sans lien externe (mieux référencé) ; mets le
  lien vers ton rapport ou ta démo en **premier commentaire**.
- **Call-to-action** : la dernière ligne invite à la discussion → plus de
  commentaires = plus de portée.
- **Cohérence avec ton profil** : le post reprend mot pour mot l'intitulé de ton
  expérience LinkedIn (« Stagiaire Fin d'Études – Architecture Data & IA »,
  « architecture Coffre-Fort Data P2P Souveraine »). Ne les reformule pas :
  c'est ce qui fait que le recruteur qui clique sur ton profil retrouve
  exactement le vocabulaire du post — et que les deux se renforcent au lieu de
  paraître être deux projets différents.
- **Positionnement Data & IA** : le bloc « agents IA distribués » est ce qui te
  différencie des autres posts PFE cyber/crypto. Si tu ne devais garder qu'une
  phrase à mettre en avant en commentaire ou en accroche de candidature, c'est
  celle-ci : *l'IA vient jusqu'à la donnée, la donnée ne sort pas.*
