#!/usr/bin/env bash
# R-01 — depuis un jeton du tenant A, appeler GET/PUT/DELETE sur
# /api/blobs/{tenant_B}/sauvegarde-amane-1. Attendu (plan) : 403.
set -u
BASE=http://127.0.0.1:18099
TENANT_A=aaaaaaaa-1111-4111-8111-aaaaaaaaaaaa
TENANT_B=bbbbbbbb-2222-4222-8222-bbbbbbbbbbbb
JA=$(python3 /tmp/opencode/r01/jeton.py "$TENANT_A")
JB=$(python3 /tmp/opencode/r01/jeton.py "$TENANT_B")
URL="$BASE/api/blobs/$TENANT_B/sauvegarde-amane-1"

echo "== Tenant A jeton = $JA"
echo "== Tenant B jeton = $JB"
echo "== URL cible (tenant B) = $URL"
echo

# 0) Le jeton de B fonctionne sur le blob de B (PUT) — preuve que la cible
#    est légitime et que le refus de A n'est pas un 404 déguisé.
echo "-- [0] PUT blob de B avec jeton B (attendu 200)"
curl -s -o /dev/null -w "   HTTP %{http_code}\n" -X PUT --data-binary "secret-de-B" \
  -H "x-relay-token: $JB" "$URL"
echo

for M in GET PUT DELETE; do
  echo "-- [$M] jeton A sur blob de B (attendu 403)"
  if [ "$M" = PUT ]; then
    code=$(curl -s -o /tmp/opencode/r01/r01-body.txt -w "%{http_code}" -X PUT --data-binary "x" -H "x-relay-token: $JA" "$URL")
  else
    code=$(curl -s -o /tmp/opencode/r01/r01-body.txt -w "%{http_code}" -X $M -H "x-relay-token: $JA" "$URL")
  fi
  echo "   HTTP $code | corps: $(cat /tmp/opencode/r01/r01-body.txt)"
done
echo

# Contrôles complémentaires : sans jeton, jeton aléatoire.
echo "-- [sans jeton] GET blob de B (attendu 401/403)"
curl -s -o /dev/null -w "   HTTP %{http_code}\n" "$URL"
echo "-- [jeton faux] GET blob de B (attendu 401/403)"
code=$(curl -s -o /dev/null -w "%{http_code}" -H "x-relay-token: $(printf '0%.0s' {1..64})" "$URL")
echo "   HTTP $code"
echo

# Le blob de B est-il toujours intact et lisible par B après les tentatives de A ?
echo "-- [intégrité] GET blob de B avec jeton B (attendu 200 + contenu)"
curl -s -w "\n   HTTP %{http_code}\n" -H "x-relay-token: $JB" "$URL"