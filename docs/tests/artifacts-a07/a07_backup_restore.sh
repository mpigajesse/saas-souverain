#!/bin/bash
# A-07 — Sauvegarde chiffrée + restauration dans une base jetable.
# S'exécute DANS le conteneur amane-postgresql du primaire (m1).
# Mécanisme identique au nœud (pg_dump puis chiffrement) ; ici AES-256-CBC
# avec une clé de test (la DEK réelle est scellée par ss-crypto ; le relais
# de l'éditeur est injoignable depuis le banc, cf. docs/tests/REPLICATION.md).
set -e
KEY='0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'

echo "== 1) base source jetable a07_src : 500 lignes =="
psql -U postgres -c "DROP DATABASE IF EXISTS a07_src" >/dev/null
psql -U postgres -c "CREATE DATABASE a07_src" >/dev/null
psql -U postgres -d a07_src -q -c "CREATE TABLE t(id serial primary key, libelle text); INSERT INTO t(libelle) SELECT 'x-'||g FROM generate_series(1,500) g;"
SRC=$(psql -U postgres -d a07_src -tAc "select count(*) from t")
echo "   compte source = $SRC"

echo "== 2) pg_dump puis chiffrement AES-256-CBC =="
pg_dump -U postgres -d a07_src -f /tmp/a07_src.sql
openssl enc -aes-256-cbc -pbkdf2 -salt -pass pass:$KEY -in /tmp/a07_src.sql -out /tmp/a07_src.sql.enc
echo "   taille clair=$(wc -c </tmp/a07_src.sql) octets ; chiffré=$(wc -c </tmp/a07_src.sql.enc) octets"
echo -n "   occurrences de 'libelle' dans le chiffré : "
if grep -a -q "libelle" /tmp/a07_src.sql.enc; then echo "FUITE TROUVEE"; else echo "0 (chiffré opaque, OK)"; fi
echo -n "   en-tête du blob chiffré (file) : "; file /tmp/a07_src.sql.enc 2>/dev/null || echo "(file absent)"

echo "== 3) restauration dans la base jetable a07_dst =="
psql -U postgres -c "DROP DATABASE IF EXISTS a07_dst" >/dev/null
psql -U postgres -c "CREATE DATABASE a07_dst" >/dev/null
openssl enc -d -aes-256-cbc -pbkdf2 -pass pass:$KEY -in /tmp/a07_src.sql.enc -out /tmp/a07_src.dec.sql
psql -U postgres -d a07_dst -q -f /tmp/a07_src.dec.sql
DST=$(psql -U postgres -d a07_dst -tAc "select count(*) from t")
echo "   compte restauré = $DST"

echo "== 4) contrôle : mauvaise clé =="
if openssl enc -d -aes-256-cbc -pbkdf2 -pass pass:mauvaisecle -in /tmp/a07_src.sql.enc -out /tmp/x.sql 2>/dev/null; then
  echo "   ANOMALIE : déchiffrement avec mauvaise clé réussi"
else
  echo "   mauvaise clé -> échec de déchiffrement (OK)"
fi

echo "== 5) nettoyage des bases jetables =="
psql -U postgres -c "DROP DATABASE a07_src" >/dev/null
psql -U postgres -c "DROP DATABASE a07_dst" >/dev/null
rm -f /tmp/a07_src.sql /tmp/a07_src.sql.enc /tmp/a07_src.dec.sql /tmp/x.sql
echo "   bases a07_src et a07_dst supprimées"

echo "== VERDICT =="
if [ "$SRC" = "500" ] && [ "$DST" = "500" ]; then echo "OK: source=$SRC restaure=$DST"; else echo "ECHEC: source=$SRC restaure=$DST"; fi