# Cross-Cycle Insights (cycles 0001–0009)

## Recurring wins

- **Changes to structure beat parameter tuning.** The big steps all changed the structure: the 24/8 topology (c0001), the frequency-aware 8-slot tail victim (c0002), the ghost-credited admission (c0004) and its cache-sized direct-mapped form (c0005), and ghost hits bypassing the stash (c0007). Sweeping a parameter inside one of these structures has usually found the peak within a cycle or two and then stalled.
- **Changes inside a bucket move the small size.** Victim choice and admission rules have reliably moved the small cache size. Topology-only changes never did.
- **Ageing works when it targets items already judged cold.** Halve-on-cap, stash-hit halving, and halving on demotion into the stash (c0006) all helped. Ageing that hits recently arrived or Front-landed items has always hurt.
- **Fixing where proven-reuse keys land beat giving them more credit.** Ghost reach and bonus both stalled until credited keys skipped the short-lived stash (c0007).
- **Protecting warm residents works better than protecting newcomers.** The contention-gated bypass (c0008) helped small-size hm_0. Exempting newcomers from the victim scan hurt every cell.
- **A change helps only where the ghost fires.** Every hm_0 gain since c0004, and the only real web_0 gain (large size, c0007), came through the ghost path.

## Recurring failures

- **Insert-position tweaks.** MRU promotion landing hurt web_0 in c0001, c0002 and c0003, alone or bundled. Gated demotion position was redundant once the tail scan reads counters. Front landing is part of how the frequency filter works.
- **Sweeps that are now used up:**
  - normal/stash ratio (peaked near 3/4)
  - tail window width (8 is the peak; both 4 and 16 lost)
  - counter cap
  - ghost size above capacity
  - ghost bonus above 1
  - fingerprint narrowing (false matches dent web_0)
  - global gate threshold (it moves the trade-off between sizes but never improves both)
- **Faster ageing or ageing triggered by eviction count** (c0004, c0007). It drains Front-landed counter-1 arrivals inside the window, and it hurts small caches the most.
- **Removing ageing points** such as the stash-hit halving (c0005). Items with high counters then stick in target, and every cell gets worse.
- **Direct scan defence on web_0.** A TinyLFU gate on promotion (c0007) never fired, because one-hit scan keys never reach it.
- **More placement options under the same cold-only rule.** Neighbour-bucket placement (c0009) failed because the neighbour shares the segment's load.
- **Learned signals without instrumentation.** The re-hit score (c0009) showed no measurable effect, and nobody could tell whether it rarely fired or was miscalibrated.
- **Bundling several changes in one variant** (c0002 v001). It hides which change caused a regression.

## Open questions

- Would a bypass gate that depends on contention or occupancy work better than a fixed one: relaxed (≤1) where the target is uncontended or large, strict (==0) where it is contended? Would a tie-break at equal counters that favours the resident help?
- Large-size hm_0 is close to flat. Is that structural? Do keys leave through paths the ghost never records?
- How often does each gate branch and ghost path fire? Instrumentation is needed before any adaptive gate is built.
- Would ageing at other boundaries help, if it targets only cold items: stash→target promotion, or a normal-tier scan that excludes recent promotions?
- Is there any web_0 lever outside the ghost path?
- The cost side is still unknown. Per-op-time and metadata-size have not been measured in any of the nine cycles, and overlay landing was unconfirmed for several variants in c0008 and c0009.

## Per-policy notes

- **Current lineage:**
  1. c0001 v002: 24/8 topology
  2. c0002 v002: frequency-aware 8-slot tail victim, halve-on-cap
  3. c0004 v002: ghost ring, admit at counter 1
  4. c0005 v001: direct-mapped ghost sized to the cache
  5. c0006 v003: halve on demotion into the stash
  6. c0007 v002: ghost hits go straight to target Front
  7. c0008 v002: contention-gated bypass (c0009 v002 is equivalent within noise)
- **Gains are shrinking.** They have been small since c0006, apart from c0007's structural jump. The current structure looks close to its limit, so a structurally different hypothesis is probably worth a slot.
- **hm_0 responds most.** It shows the largest gains and the largest regressions. The small-size hm_0 cell is the most sensitive to frequency and admission changes.
- **web_0 has barely moved.** Its one real gain came at the large size through the ghost bypass. Large-size web_0 tail windows bottom out at counter 1, and that is where the open gap between the two sizes sits.
- **The best setting depends on cache size.** The bypass/stash and gate-threshold trade-offs flip between the small and large sizes. Future rules likely need to adapt to size or contention.
