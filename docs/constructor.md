# Constructor mode: a von Neumann substrate for Abiogenesis

Status: proposal, not built. It's the candidate for the stop rule's "rethink the substrate"
step, built only if experiments 2 (genome ocean) and 3 (learned mutation) both fail.

## Why

Abiogenesis, Tierra and Avida all reproduce by **self-inspection**: a creature reads its own
body cell by cell and writes the copy. The genome *is* the phenotype. Von Neumann described
a second way: **build from a description**. A constructor reads a tape and builds what it
describes; a copier copies the tape blindly. Life uses the second way (DNA is copied by
polymerase and read by ribosomes).

Two problems in our runs trace back to self-inspection:

1. **Copy cheating.** A creature that runs its own copy loop can cheat the copying. We
   patched four versions: oversized junk children, children filled from outside the body,
   skipping the last eighth, and copying the first half twice. Each one destroyed the
   novelty measure until found, and the next one may already be evolving.
2. **Most evolution goes into the copy loop.** In Tierra-style worlds, selection mostly
   tunes replication speed (smaller, faster, parasitic), not what creatures do in the world.
   That's the plateau we keep measuring: class 2, bounded novelty.

## What's been tried (checked 2026-10-16)

- **Clark, Hickinbotham & Stepney 2017** (Stringmol, J. R. Soc. Interface): genome, copier
  and expressor (ribosome-like codon table) all on strings. The copier and expressor evolved
  and swapped, sometimes changing what the genome *means* without changing the genome. But:
  - every run went extinct (mean about 6.6M steps), by "bureaucratic death": a diversity
    explosion, then collapse;
  - the expressor's codon table lost opcodes over time;
  - no claim of open-ended evolution or growing complexity;
  - they note that earlier Tierra and Avida universal-constructor attempts were brittle and
    **collapsed back into self-copiers**.
- **"Self-Reproduction and Evolution in Cellular Automata: 25 Years after Evoloops" (2024,
  *Artificial Life* 31(1))** lists as open problems:
  - Darwinian evolution in a *population* of von Neumann constructors (never shown in CA);
  - the fact that every evolving self-replicator system ends dominated by one or a few
    species. Dynamic environments help, but "may not work for indefinitely long terms".
- Yinusa & Nehaniv 2011: tape mutations in von Neumann's own automaton can raise complexity,
  but they were hand-engineered, not evolved.
- Altenberg 1994/95, "constructional selection": when genomes grow by adding genes that
  survive selection, the genotype-phenotype map becomes smoother and more evolvable. That's
  an argument for letting the tape grow by gene duplication rather than point edits alone.

Lessons we take from this:
- Don't let creatures run their own copying, or they collapse back to self-copiers.
- Watch for diversity explosions followed by collapse, and for lost codons.
- A constructor alone doesn't cure single-species dominance. We still judge by Channon's
  test, and the ocean stays as the changing environment.

## Design

**Stage 1: blind copying and development (tests the core idea)**
- A creature is a **tape** (inherited bytes), a **body** (VM code), memory and energy.
- New op `reproduce`. The world copies the tape blindly, with mutations, insertions,
  deletions and **gene duplication** (copy a random segment next to itself, Altenberg). It
  then builds the child's body by **translation** and charges the parent energy in
  proportion to tape length. `mal`, `divide` and writing into a child are switched off, so
  no creature code ever does copying, and copy cheating can't exist.
- **Translation** (fixed in Stage 1): the tape is read in 1-byte codons. Most codons map to
  an op. A few are structural:
  - *stop*: skip the rest of a gene, giving non-coding regions where neutral drift happens;
  - *repeat-next-gene*: express a gene twice, a cheap route to modularity.
- Bodies keep the current VM, ocean, memory and energy economy, so what's under selection
  is what bodies *do*: predict the ocean, earn energy, infect.
- **Novelty measure:** a Channon component is the **body** (expressed phenotype), so silent
  tape changes don't count as novelty. We also log tape genotypes to watch the hidden layer.

**Stage 2: an evolvable genetic code (semantic closure, partly)**
- The first 64 bytes of the tape are a **code table** that says which op each codon
  produces. It's inherited and mutated like the rest, so the *meaning* of the genome can
  evolve, as in the Stringmol result.
- Track code-table coverage, to catch the lost-opcode failure.

**Stage 3: creature-run translation (full von Neumann), only if 1 and 2 show promise**
- A creature's own code reads the tape and writes the body, so the constructor itself evolves.
- Copying stays blind and done by the world, which blocks the known collapse into
  self-copiers.

## Experiment

Same rule as every other mechanism, fixed before any run:
- **Arms:** Stage 1 vs the lean baseline (`LB_base`): seeds 1–10, 20,000 epochs, energy
  economy, enwik8 ocean.
- **Win** = Channon class 3 in at least 8 of 10 pairs, judged by `abiogenesis compare`.
- **Extra checks:**
  - no extinctions (Stringmol's runs all died);
  - no "explode then collapse" in diversity;
  - ocean prediction at least as good as the baseline.
- Stage 2 is tested only if Stage 1 wins or comes close (6–7 of 10). Stage 3 only if
  Stage 2 wins.

## Risks

- **Bodies may stop doing anything interesting** once there's no copy loop to tune. Then
  selection depends entirely on the ocean and energy rewards, which have plateaued before.
  Stage 1 could plateau for the same reason the ocean did.
- **The pairing is weaker**: different substrates with the same seeds aren't matched as
  tightly as one-feature changes. The test is still fair, because Channon's measure is
  normalized against chance.
- **Defining "component" as the body** is a choice. Tape-level novelty is logged too, so we
  can see whether the answer depends on it.

## Size

Stage 1 is a new reproduction path (`reproduce`, tape copying, translator) plus config,
census and verify support. It's smaller than the energy economy was. One batch (about 3 hours)
judges it.
