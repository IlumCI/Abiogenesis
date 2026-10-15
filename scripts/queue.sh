#!/usr/bin/env bash
# Runs experiment arms: seeds 1..10 per arm, at most $MAX worlds at a time, niced.
#
#   scripts/queue.sh "A=--no-tasks --ocean oceans/enwik8|B=--no-tasks --ocean genomes" AB_DONE
#
# Env: EXE (default: a frozen copy of target/release/abiogenesis), EPOCHS (20000),
# MAX (default: 3/4 of the CPU threads). Detach with: nohup scripts/queue.sh ... &
set -u
cd "$(dirname "$0")/.."
arms=$1
done_name=${2:-DONE}
epochs=${EPOCHS:-20000}
max=${MAX:-$(( $(nproc) * 3 / 4 ))}
(( max < 1 )) && max=1
if [[ -z ${EXE:-} ]]; then
    # Freeze the binary, so rebuilding mid-batch can't change half the runs.
    EXE=runs/abiogenesis-$done_name
    mkdir -p runs && cp target/release/abiogenesis "$EXE"
fi
mkdir -p runs
IFS='|' read -ra arm_list <<< "$arms"
for s in $(seq 1 10); do
    for arm in "${arm_list[@]}"; do
        name=${arm%%=*}_$s
        extra=${arm#*=}
        while (( $(jobs -rp | wc -l) >= max )); do wait -n; done
        rm -rf "runs/$name"
        # $extra is split on spaces on purpose.
        nice -n 10 "$EXE" run --epochs "$epochs" --seed "$s" --economy energy --out "runs/$name" $extra \
            > "runs/$name.out" 2> "runs/$name.err" &
    done
done
wait
date > "runs/$done_name"
