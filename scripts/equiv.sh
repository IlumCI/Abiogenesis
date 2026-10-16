#!/usr/bin/env bash
# Check that a new binary replays every engine path exactly like an old one: runs a set of
# configurations for EPOCHS epochs with both and compares their chain.log files.
#
#   scripts/equiv.sh OLD_BINARY NEW_BINARY [EPOCHS]
#
# A speed change must pass this before it is committed (AGENTS.md: hashes unchanged).
set -u
cd "$(dirname "$0")/.."
old=$1 new=$2 epochs=${3:-300}
out=runs/equiv
mkdir -p "$out"
O="--ocean oceans/enwik8"
configs=(
  "plain|--no-tasks --economy energy $O"
  "reef_allops|--no-tasks --economy energy $O --all-ops --reef"
  "v4_mem2_learner|--no-tasks --economy energy $O --v4 --energy-total 8000000 --mem2 --ancestor ancestors/learner.tasm"
  "streams_debt|--no-tasks --economy energy $O --v4 --energy-total 8000000 --mem2 --streams 64 --debt"
  "free_hgt_cost|--no-tasks --economy energy $O --v4 --energy-total 8000000 --mem2 --hgt --free-copy --birth-cost 10"
  "tasks_merit|$O"
  "social_capitalist|--no-tasks --economy energy $O --social --regime capitalist"
  "genomes_learnedmut|--no-tasks --economy energy --ocean genomes --learned-mut 25"
)
fail=0
for c in "${configs[@]}"; do
  name=${c%%|*}; flags=${c#*|}
  for which in old new; do
    exe=$old; [[ $which == new ]] && exe=$new
    rm -rf "$out/${name}_$which"
    # $flags is split on spaces on purpose.
    nice "$exe" run --epochs "$epochs" --seed 3 --out "$out/${name}_$which" $flags > /dev/null 2>&1 &
  done
done
wait
for c in "${configs[@]}"; do
  name=${c%%|*}
  if cmp -s "$out/${name}_old/chain.log" "$out/${name}_new/chain.log"; then
    echo "same   $name ($(wc -l < "$out/${name}_new/chain.log") epochs)"
  else
    echo "DIFFER $name"; fail=1
  fi
done
exit $fail
