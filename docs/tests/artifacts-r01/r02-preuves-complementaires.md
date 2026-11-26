# R-02 — Preuves complémentaires (coupure TCP à 20 %, 500 Mo)

## 1. Coupure à 20 % (r02-kill-sortie.txt)
envoye=104857600 octets (20.0 % de 524288000) en 0.11s
connexion coupee (RST) a 20%

## 2. État du disque après coupure (find blobs -type f)
Seul subsistait le blob de test R-01 du tenant B (11 octets, déposé
légitimement avant R-02). AUCUN fichier résiduel du tenant C
(cccccccc-…), cible du transfert avorté :
    -rw-r--r-- 1 jose jose 11 ... blobs/bbbbbbbb-.../sauvegarde-amane-1
    (fin find)

## 3. GET après coupure (attendu 404)
    HTTP 404

## 4. Log relais après coupure
Aucune ligne « Blob stocke » pour le tenant C : le relais n'a jamais écrit
(le corps complet n'a pas été reçu, `Bytes` non extrait, handler non appelé).

## 5. Ré-émission (le nœud réémet) — PUT complet 500 Mo
    PUT HTTP 200  temps=1.270129s
    GET HTTP 200  taille=524288000 octets
    -rw-r--r-- 1 jose jose 524288000 ... blobs/cccccccc-.../sauvegarde-amane-1

Conclusion R-02 : aucun fichier résiduel, GET=404, ré-émission acceptée et
intégrale (500 Mo relus). PASS.