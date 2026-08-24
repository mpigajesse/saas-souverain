#!/usr/bin/env bash
# Vérification de régression des micro-benchmarks (job CI `benchmark`).
#
# Lance les benchmarks Go des chemins chauds (replication, sans etcd) et
# compare la MÉDIANE des ns/op à la baseline docs/bench/baseline.json.
# Seuil = baseline × micro_tolerance_mult (bruit CI : pas de blocage fin).
# Sortie : liste OK / REGRESSION ; code de sortie ≠ 0 si regression.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE="$ROOT/docs/bench/baseline.json"
OUT="$(mktemp)"
trap 'rm -f "$OUT"' EXIT

echo "== benchmarks (count=3, médiane vs baseline) =="
(cd "$ROOT/orchestrator-go" && go test ./replication/ -run '^$' -bench=. -benchmem -benchtime=1s -count=3) > "$OUT" 2>&1 || { cat "$OUT"; exit 1; }

fail=0
while IFS= read -r bench; do
  baseline="$(jq -r ".micro_ns_op[\"$bench\"] // empty" "$BASE")"
  [ -n "$baseline" ] && [ "$baseline" != "null" ] || continue
  tol="$(jq -r '.micro_tolerance_mult' "$BASE")"
  median="$(awk -v b="$bench" 'index($1,b)==1 && match($1,/^[A-Za-z]*-[0-9]+$/) {print $3}' "$OUT" | sort -n | awk '{a[NR]=$1} END{print (NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2}')"
  [ -n "$median" ] || { echo "MANQUANT $bench : aucun résultat (a-t-il été renommé/supprimé ?)"; fail=1; continue; }
  limit="$(awk -v b="$baseline" -v t="$tol" 'BEGIN{printf "%.2f", b*t}')"
  if awk -v m="$median" -v l="$limit" 'BEGIN{exit !(m>l)}'; then
    echo "RÉGRESSION $bench : médiane ${median} ns/op > seuil ${limit} (baseline ${baseline})"
    fail=1
  else
    echo "OK $bench : ${median} ns/op (baseline ${baseline}, seuil ${limit})"
  fi
done < <(jq -r '.micro_ns_op | keys[]' "$BASE")

exit "$fail"