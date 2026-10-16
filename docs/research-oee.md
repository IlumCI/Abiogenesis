# What drove sustained novelty elsewhere (review, 2026-10-17)

A literature review done after the reef, bigger memory, streams and transfer all failed
Channon's test. Most citations were checked by search or in full text; "(unverified)" marks
the rest.

## Artificial life

| System | Mechanism the authors credit | Evidence |
|---|---|---|
| Geb (Channon 2001 ECAL; 2006 GPEM 7:253) | Biotic selection with no fixed fitness. Agents can kill, and every birth is sexual with crossover. Neural nets grown by an L-system. | The only system to pass the shadow test, counting production rules (genes). Channon 2019 (*Artificial Life* 25:134): maximum complexity grows only logarithmically, and only when world size and the per-individual cap are scaled together. |
| Avida host–parasite (Zaman et al. 2014, *PLoS Biol* 12:e1002023) | Frequency-dependent coevolution | EQU evolved in 17/50 coevolving populations, 0/50 without parasites. Hosts lost complexity when parasites were frozen or removed. Well-mixed world. |
| Avida predator–prey (Wagner, Zaman, Dworkin & Ofria, arXiv 1310.1369) | An attack instruction; predators must eat several prey to reproduce | Predator behaviour improved linearly over about 19,500 prey generations, with no plateau shown. |
| Tierra (Ray 1991) | Coevolution: parasites, immunity, hyper-parasites | The arms race was not sustained. |
| Avida division of labour (Goldsby, Dornhaus, Kerr & Ofria 2012, *PNAS* 109:13686; 2014, *PLoS Biol*) | Task-switching costs; costs of doing mutagenic work | Major transitions occur. Open-endedness was not measured. |
| Chromaria (Soros & Stanley 2014) | Four conditions, including a non-trivial minimal criterion for reproduction | Stagnates when any one is removed. |
| Hickinbotham, Stepney & Hogeweg 2021 (*R Soc Open Sci* 8:210441) | Parasites | Parasites emerge inevitably and drive complex replication, but wipe out well-mixed populations unless space or compartments protect them. |

## Biology

- **Cambrian explosion.** The best-supported driver is ecological escalation through
  predation. Oxygen was a permissive condition: Sperling et al. 2013, *PNAS* 110:13446, found
  fewer carnivores at low oxygen. The developmental toolkit was necessary but older than the
  explosion. Eyes as the trigger ("light switch") has little support.
- **Multicellularity evolves fast under predation:** Herron et al. 2019, *Sci Rep* 9:2328,
  saw it in 2 of 5 algae lines within about 750 generations, and in no controls. It also
  evolves fast under selection for size: Ratcliff et al. 2012, *PNAS* 109:1595.
- **Lane & Martin 2010** (energy per gene) is disputed by Lynch & Marinov 2015 and 2017.

## Plateau causes we hadn't covered

- **Fixed physics allows only exploratory novelty** (Taylor 2019, *Artificial Life* 25).
  Hickinbotham et al. 2016 argue that the layer that can't evolve (here, the copy rules
  and the interpreter) should be kept minimal.
- **Capacity bounds results** (Channon 2019). Our 512-cell size cap and 262k-cell soup may
  limit any mechanism.
- **Asexual hitchhiking** blurs gene-level counts; sex fixes this, which is part of why
  Channon's components work.

## Ranked for a redesign

1. **Predation**, with local interaction and local birth so predators can't wipe out a
   well-mixed world.
2. **Obligatory sexual recombination**, or much stronger transfer. Our transfer lead points
   the same way.
3. **Death by starvation as a minimal criterion**, with several limiting resources
   (Chow et al. 2004, *Science* 305:84). Moderate evidence.

No mechanism has been judged by Channon's test on its own, only bundled together in Geb.
