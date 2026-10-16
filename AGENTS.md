# AGENTS.md: handoff and working rules

Read this, then `README.md`, then `docs/constructor.md`. This file carries what isn't in
the code: where the work stands, how experiments are judged, what went wrong before, and
how the owner likes to work. Last updated 2026-10-16, when work moved from a Windows PC
to Linux.

## Where things stand

- **Goal:** open-ended evolution, meaning novelty that doesn't plateau, measured by
  Channon's shadow-resetting test (`src/channon.rs`). Any token or product layer comes only
  after a real result.
- **Known:** baseline worlds show bounded novelty (class 2), the Tierra/Avida plateau.
- **Experiment 1, reef** (`--reef`): **FAIL**. It lost every pair that finished; mostly
  judged before the full-copy fixes below, but the verdict was settled early.
- **Experiments 2 and 3 (genome ocean, learned mutation) are shelved, not rerun**
  (2026-10-16, agreed with the owner). An audit found reasons why their verdict on this
  build would mean little:
  - **Ocean reward is mostly lost.** Score is paid at the next epoch's payout, and only if
    the mother is still alive.
  - **The 4-op learner can't beat a constant guess.** See the ceilings below.
  - **Learned mutation has a confound.** It often writes the op being copied, which lowers
    its effective mutation rate below the control's.
  - **The genome ocean may reward homogeneity.**

  Their configs stay in "Rerun the experiments" below, in case a fixed baseline makes them
  worth judging.
- **Ocean ceilings** (`scripts/ceiling.py`, bits per guess above the best constant guess,
  20 MB samples):

  | Ocean | 4-op learner | Order-1 best | Order-2 | Order-3 |
  |---|---|---|---|---|
  | enwik8 | +0.01 | +0.40 | +0.66 | +1.00 |
  | text8 | −0.21 | +0.24 | +0.32 | +0.71 |
  | wikiprose | −0.16 | +0.29 | +0.35 | +0.71 |

  - Live creatures score about 1.2 bits/guess, below the constant guess (1.6).
  - Learning is invisible to selection until memory allows order-2 contexts (64 KB).
  - `oceans/wikiprose` is enwik8 with markup stripped (`scripts/strip_wiki.py`, sha1
    `230a99c960345028ae0e4acbde8878db2092d55a`).
- **Gene-level meter** (`src/genes.rs`, census columns `gene_*`). It applies Channon's test
  with genes as components, as in Geb, which is the only system to pass the test.
  - Genes are code between labels, counted once executed.
  - A null shadow (`gene_null_new`) measures how many genes chance alone marks
    significant.
  - At 1,500 epochs, real significant genes outnumber null ones about 65:1.
  - Caveat: the world is asexual, so every gene in a winning genome hitchhikes. The gene
    count overstates how many distinct adaptations there are.
  - `compare` prints a gene-level verdict next to the genotype one. **The gene verdict is
    official** (fixed 2026-10-16, before any data); the genotype verdict is a secondary
    readout.
- **Plan from here** (agreed 2026-10-16; the owner gates each step):
  1. Baseline v4: score paid at divide, no free bits, random ocean start, ocean chosen
     from the ceilings.
  2. Fast screening: small and short, then confirm long.
  3. Then, one at a time:
     - lift the predictor cap (memory, reading soup cells);
     - frequency-dependent ocean income;
     - reef v2, with involuntary emission and mixed cooperation and competition;
     - constructor mode last.
- **Stop rule** (agreed with the owner): if experiments 1–3 all fail, stop adding mechanisms
  and rethink the substrate. The candidate is constructor mode (`docs/constructor.md`):
  - von Neumann's tape-and-constructor design;
  - creatures never copy themselves.

  **Discuss it with the owner before building any of it.**
- A mechanism that wins joins the core and gets a 100,000-epoch confirmation run.

## Rerun the experiments (Linux)

```
cargo build --release
mkdir -p oceans && cd oceans && curl -LO https://mattmahoney.net/dc/enwik8.zip && unzip enwik8.zip && cd ..
sha1sum oceans/enwik8        # 57b8363b814821dc9d47aa4d41f58733519076b2
nohup scripts/queue.sh "LB_base=--no-tasks --ocean oceans/enwik8|GO_gen=--no-tasks --ocean genomes|LM_learn=--no-tasks --ocean oceans/enwik8 --learned-mut 25" V3_DONE &
# check the settings took, right after launch (a launch bug once silently dropped the ocean):
grep -E '^(ocean|learned_mut|lean)=' runs/LB_base_1/config.txt runs/GO_gen_1/config.txt runs/LM_learn_1/config.txt
# when runs/V3_DONE exists:
target/release/abiogenesis compare runs/GO_gen runs/LB_base
target/release/abiogenesis compare runs/LM_learn runs/LB_base
```

Cost: 30 runs × 20,000 epochs.
- On the 16-thread Windows PC, 12 at a time ran about 140 epochs/min per world, roughly
  5–6 hours a batch.
- On this Linux machine a single world runs the whole 20,000 epochs in about 20 minutes,
  1,000 epochs/min at population 3,300 (measured 2026-10-16). Expect a batch in about 2
  hours.
- `queue.sh` defaults to 3/4 of the threads (`MAX=` to change).

**Check this build matches the Windows build** (seed 1, 100 epochs, epoch-99 line of
`chain.log`):

| Command | Expected hash |
|---|---|
| `run --epochs 100 --seed 1 --no-tasks --economy energy --ocean oceans/enwik8` | `192577de66da806e08b2b113f4d51d6c4dd9a7c127eb486597016ecced0edf65` |
| the same plus `--all-ops --reef` (the pre-lean engine) | `6c829caf0c65f879bccda078046f229e3d4bd7f6cd8e9162699458f02ba6c430` |

The simulation uses only integer and basic float math, so they should match. If they don't,
find out why before trusting any comparison with Windows-era results.

## How experiments are judged (fixed, don't change after seeing data)

- **One mechanism per experiment.** Same build for both arms, seeds 1–10, 20,000 epochs,
  `--economy energy --no-tasks`, and the enwik8 ocean unless the ocean is the thing tested.
- **Win** = `abiogenesis compare` says `verdict (genes, official): ... WIN`, which needs at
  least 8 of 10 pairs where, counting genes (`gene_*` columns) as components,
  - cumulative gene activity is still growing (last quarter's mean above the second
    quarter's) and above the control's;
  - second-half new significant genes are at least half the first half's, and more than
    the control's;
  - second-half new significant genes are at least 10× what the arm's own null shadow
    marks significant by chance (`gene_null_new`).

  The genotype verdict (`norm_*`) is printed too, as a secondary readout. Fixed
  2026-10-16, before any data.
- **Plain-language target** (what counts as breaking the plateau): in a 100,000-epoch
  run, new significant genes keep appearing at a rate that doesn't fall away, and stay
  10× above chance.
- **Read the `!` warnings before the verdict:**
  - *meltdown* (nearly every creature a distinct genotype);
  - *extinct*;
  - *ledger gap nonzero*.

  A melted-down control reads zero novelty and hands the other arm a fake win.
- **Every engine change:**
  - a 300-epoch smoke run with `ledger_gap` 0 in every epoch;
  - `abiogenesis verify DIR`;
  - the `--all-ops --reef` hash above unchanged, unless the change is meant to alter the
    old engine.

## Lessons (each cost a batch)

- **Creatures cheat the copy rule.** Any slack in "a child must be a copy" gets exploited,
  and the result is junk in every child, every child a new genotype, and novelty reading
  zero. Four variants so far:
  - oversized children padded with junk;
  - copying from outside the body;
  - skipping the last eighth;
  - copying the first half twice.

  Lean mode now requires every distinct cell copied from the mother's own body
  (`child_seen` in `src/world.rs`). Expect a fifth variant, and suspect one whenever the
  diversity/population ratio climbs toward 1.
- **Earlier extinctions and their fixes:**

  | Problem | Fix |
  |---|---|
  | Running a dead creature's leftover code | Body decay |
  | Absolute CPU time stalled worlds | Relative CPU share |
  | Frozen giant worlds | Old-age death, size cap 512 |
  | Reaping to make room | Removed |
  | Energy-economy collapses from a starvation rule | Broke creatures sleep instead of dying |
  | Rent above income | Capped at 80% of land yield |
- **The original Bedau–Packard meter was biased.** Use Channon's test.
- **Stacking mechanisms got us lost** (51 ops, nothing proven). Test one hypothesis at a
  time; unproven ops stay out of the mutation alphabet (`--all-ops` restores them).
- **Out of scope until something wins:** live world, viewer, token mechanics, regimes and
  politics, social ops.

## Process

- Too many simulations plus a build crashed the Windows PC. Run batches niced, at most about
  3/4 of the threads, and don't build in the same tree while a heavy batch runs (`queue.sh`
  freezes a copy of the binary).
- Runs are pure functions of config, seed and feed. `verify` replays them exactly.
- `runs/` and `oceans/` are gitignored. Results go into this file or the README as
  summaries, not raw data.

## Working with the owner

- Short replies. No boilerplate, no generic security or risk lectures, no hype. Be straight;
  if something is a pitch rather than a fact, say so.
- Don't inflate time estimates. They've been wrong in both directions; measure, then say.
- Autonomy is welcome: keep working without check-ins, but ask before big new builds or
  direction changes. Risks are fine.
- Cheap infrastructure: this machine and free tiers. Colab (TPU) only for urgent heavy
  one-off batches. No large language models inside the project.
- **Commits:** no AI-assistant trailers, co-author lines, "generated with" notes or tool
  names, in commits, PRs or files. Author as `CI-DEV <154627941+IlumCI@users.noreply.github.com>`.
  Check `git log --format=%B` before pushing.
