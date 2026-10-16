# Abiogenesis

A world where programs evolve for real. Descended from Thomas Ray's Tierra (1991) and
Avida, opened up to the outside: every epoch, 32 bytes of entropy arrive and become weather,
and creatures can swim through real data and earn their living by predicting it.

The goal is open-ended evolution: novelty that keeps coming instead of the plateau every
Tierra-style world has hit so far. Progress is judged by Channon's shadow-resetting test
(below), not by eye.

## The world

- **Soup**: 2^18 cells of memory. Creatures are runs of instructions that copy themselves.
  The reaper keeps it 80% full. There is no fitness function.
- **Seed**: Ray's 80-cell replicator plus a 40-cell scratch region at the top of its
  reproduction loop, 124 cells in all (`abiogenesis ancestor`). Seed your own with
  `--ancestor FILE.tasm`.
- **Weather**: the feed moves a sun around the soup (sunlit creatures run 2x faster) and
  sets how hard cosmic rays hit. The `sense` op points a creature's children toward the sun.
- **The ocean** (`--ocean FILE`): a sea of real data the creatures know nothing about.
  `guess` declares the next byte, `dive` reads it. Guesses are scored bit by bit (bits right
  minus 4), and every step of better prediction pays. Blind guessing is not quite zero:
  each cycle's total is floored at 0, and a constant guess gets text's shared high bits
  free (about +1.6 per guess on enwik8; `scripts/ceiling.py`). `--v4` fixes this: a guess
  earns only the bits it gets beyond the ocean's best constant guess, so blind or constant
  guessing earns nothing; the score is paid in energy at divide; and the founder starts at a
  random place in the sea. A
  child swims on from where its mother stopped, so lineages can't memorize, only learn
  patterns. The file's hash goes in the config; `verify` refuses another sea.
  `--ocean genomes` makes the sea a sample of the living population's own genomes, rebuilt
  every epoch.
- **Memory**: a private 256-byte memory (`load`/`store`) that children inherit. `dive` leaves
  the previous byte in dx, so a 4-op learner (`store load guess dive`) exists.
- **Energy economy** (`--economy energy`): a fixed total supply. Every instruction costs one
  unit back to a commons pool, income is paid by land and prediction score, and nothing is
  ever minted. The ledger is audited exactly every epoch (`ledger_gap` in the census).
  `--regime feudal|capitalist|evolved` adds property in land.
- **Red Queen**: `infect` takes CPU time from a nearby creature (on by default, `--no-infect`).
- **Logic tasks**: Avida's nine logic functions on feed-derived inputs pay CPU time
  (Lenski et al. 2003), with modularly varying goals (Kashtan & Alon 2005). `--no-tasks`
  turns them off.
- **Opt-in mechanisms**, each failed or untested so far: `--reef` (creatures emit streams
  others can predict), `--social` (`give`, `mate`, `learn`, `teach`, `join`).

## The lean core

Mutation only draws from the instructions of features that are switched on, and running a
switched-off instruction is an error. With all 51 ops in the alphabet, any one op was too
rare for evolution to find. `--all-ops` restores the old engine bit for bit.

Lean also requires every cell of a child to be copied from the mother's own body. Without
that, lineages learned four ways to leave leftover junk in their children:
- oversized children padded with junk;
- copying from outside the body;
- skipping the last eighth;
- copying the first half twice.

Every child was then a "new" genotype, and the novelty measure read zero.

## Measuring novelty

- **Channon's test** (ALife VIII, 2002): after every census a shadow population restarts
  from the real one and replays its births and deaths with random parents and victims. A
  genotype gains activity only by outlasting what chance would have kept.
- **Gene-level test** (`gene_*` columns): the same method with genes as components, as in
  Channon's Geb. A gene is the code between two labels, counted once it has run.
  `gene_null_new` is a second shadow scored against the first: what chance alone marks
  significant.
- **Bounded vs unbounded**: bounded novelty (class 2) means normalised activity levels off;
  unbounded (class 3) means it keeps growing and new significant genotypes keep arriving.
- **`abiogenesis compare A B`** judges runs `A_1..A_10` against controls `B_1..B_10`. A
  mechanism wins if, in at least 8 of 10 paired seeds:
  - its normalised activity is still growing (last quarter above second quarter) and
    above the control's;
  - its second half produces at least half as many new significant genotypes as its
    first half, and more than the control's.
- Also tracked: Bedau & Packard activity, the MODES metrics, and ocean prediction bits.

## Status

| Experiment | Result |
|---|---|
| Baseline worlds | Bounded novelty (class 2), the Tierra/Avida plateau |
| 1. Reef | FAIL (lost every completed pair) |
| 2. Genome ocean | Shelved: confounded on the old baseline (see AGENTS.md) |
| 3. Learned mutation (`--learned-mut 25`: a quarter of mutations drawn from an order-2 model of genomes that persisted) | Shelved, same reason |
| Baseline v4 (`--v4`) | Done. Under fair scoring almost no creature guesses: the old ocean reward was free bits |

If the next experiments fail too, the next step is a new substrate: von Neumann's
description-and-constructor design instead of self-copying. See `docs/constructor.md`.

## Use

```
cargo build --release
abiogenesis run --epochs 600 --seed 7 --out runs/a
abiogenesis run --epochs 20000 --seed 1 --no-tasks --economy energy --ocean oceans/enwik8 --out runs/sea_1
abiogenesis run --feed "cmd:curl -s https://blockchain.info/q/latesthash" --out runs/live
abiogenesis verify runs/a
abiogenesis compare runs/X runs/control
abiogenesis islands ...   # several worlds with migration
```

Oceans aren't in the repo. enwik8 (100 MB of Wikipedia, the Hutter Prize file) is at
https://mattmahoney.net/dc/enwik8.zip; unzip it into `oceans/`.

`scripts/queue.sh` (Linux) and `scripts/queue.ps1` (Windows) run a 10-seed batch of one or more experiment arms, a fixed number at
a time, at below-normal priority.

Outputs per run: `report.html`, `census.csv`, `events.log`, `genebank/*.tasm`, `config.txt`,
`feed.log` and `chain.log` (used by `verify`).

### Viewer

`viewer/index.html` draws the soup (every cell, coloured by genome length; creatures
predicting the ocean glow) beside the cumulative count of new significant genes against
chance, population, readers and prediction bits. It reads frames that any `run`, `serve`
or `verify` writes when `ABIO_FRAMES` is set; frames only read the world, so hashes are
unchanged.

```
# live: watch a world as it runs
ABIO_FRAMES=runs/w1/frames ABIO_FRAME_EVERY=5 abiogenesis run --epochs 20000 ... --out runs/w1
python3 -m http.server 8000      # from the repo root
# open http://localhost:8000/viewer/#live=../runs/w1/frames

# film: replay a finished run, pack it, render an MP4 (needs ffmpeg and playwright-core)
ABIO_FRAMES=runs/film ABIO_FRAME_EVERY=10 abiogenesis verify runs/w1
python3 scripts/pack_film.py runs/film runs/film.bin --label "seed 1"
node scripts/render_film.mjs runs/film.bin runs/film.mp4
# or open http://localhost:8000/viewer/#film=../runs/film.bin (add &at=end for a still)
```

The page's Record button saves a .webm of whatever it is showing.

Replay: a run is a pure function of its config, seed and feed. `verify` re-runs it and checks
every epoch's hash. Speed: free space is tracked one bit per cell, and template searches are
cached per genotype. `cargo build --release --features prof` prints where the time goes.
