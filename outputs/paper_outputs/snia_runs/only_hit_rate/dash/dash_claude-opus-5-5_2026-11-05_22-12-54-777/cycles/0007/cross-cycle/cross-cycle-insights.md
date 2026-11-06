# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Lineage:** 24/8 normal/stash topology (c1) → frequency-aware 8-slot LRU-tail victim (c2) → ghost-credited stash admission (c4) → direct-mapped ghost sized to the cache (c5) → halving on demotion into the stash (c6) → ghost hits admitted straight to target Front (c7). Every Winner made one isolated change on top of the previous parent.
- **Single-lever, isolated variants win.** Bundled changes (c2 v001) gave results that could not be attributed to any one part. Every Winner changed exactly one thing.
- **Changes inside a bucket move the small cache size, and topology changes don't.** The victim choice and admission credit lifted the small size, while the topology split helped mostly at the large size.
- **The ghost is the main driver for hm_0.** Admission-side memory of recently dropped keys moved hm_0 when changes to recency position never did. It also produced the only real web_0 gain, at the large size in c7.
- **Structural fixes beat parameter tuning.** The largest ghost-line gain (c7) came from changing *where* credited keys land (skipping the short-lived stash), after ghost reach and bonus size had both stalled.
- **Ageing items that are already judged cold is safe.** Halving at demotion into the stash helped (c6). Halving on stash hits is load-bearing and must stay (c5).

## Recurring failures

- **Insert-position tweaks once the tail scan reads counters.** MRU promotion hurt web_0 both bundled (c2) and alone (c3). Gated demotion was redundant with the scan. Front landing is part of how the filter works.
- **Sweeping the scan window.** Window 8 is the peak on both sides: narrower loses the frequency filter (c3), and wider turns the policy into coarse LFU and loses recency protection (c4).
- **Faster or eviction-count-driven ageing.** Decrementing on every eviction (c4) and CLOCK-style decay of tail survivors (c7) drained the Front-landed, newly credited items inside the window. Both hurt small-size hm_0 the most. Front landing and ageing are coupled; this has been confirmed in five cycles.
- **Removing ageing.** Dropping the stash-hit halving (c5) made items stick and hurt every cell.
- **Normal/stash ratio past 3/4.** Shrinking the stash starves promotion, because with admit-everything-to-stash the stash is the only way into target (c2).
- **Tuning ghost reach or bonus size.** A larger table, a 2× table, and credits of +2 or remembered+2 have all been exhausted (c5–c6). A narrower fingerprint costs web_0 through false matches.
- **Scan defence aimed directly at web_0.** The TinyLFU gate on stash→target promotion did nothing (c7): one-hit scan keys never reach the gate, and target-tail counters sit near 1.
- **Unmeasured cost.** Per-op-time and metadata-size have been missing for all seven cycles, so no cost/benefit trade-off has ever been judged.

## Open questions

- At the small size, can ghost-admitted keys in target be protected long enough to be re-hit? Options: land them slightly deeper than Front, or briefly exempt them from the tail scan.
- Should the stash bypass be selective, for example by cache size or by how recent the ghost entry is, to avoid harmful demotion cascades in a crowded target?
- Are there other paths that still send proven-reuse keys through the stash (for example demoted items with prior target hits) and would benefit from direct placement?
- Does large-size hm_0 lose reuse through exit paths the ghost never records?
- Would ageing at other cold-judgment boundaries help, for example at stash→target promotion, or tail-scan ageing that skips recently promoted slots?
- Gains keep shrinking. Is the stash/target/ghost structure close to its limit, and is there a structurally different lever for web_0?
- What do the scan, the ghost lookup, and the cascade path actually cost in per-op-time and metadata-size?

## Per-policy notes

- **Current incumbent (two-tier stash/target with frequency tail scan and ghost):** the parameters that are spent are the split (3/4), the window (8), the ageing (halve on cap, on stash hit, on demotion), and ghost reach and bonus. The remaining room is in how ghost-admitted keys are placed and protected inside target.
- **hm_0:** the most responsive trace in both directions. Small-size hm_0 is where the frequency filter and the ghost do the most, and it is the most sensitive to ageing mistakes. Large-size hm_0 first moved through ageing at demotion (c6) and then through the stash bypass (c7).
- **web_0:** mostly unaffected by any lever. It reacts only when the ghost fires often enough (large size, c7). It regresses slightly under MRU landing, a narrow fingerprint, and over-credited scan keys.
- **Size dependence:** benefits from the bypass and from target residency scale with target capacity. Ageing driven by eviction count hits the small size hardest, which suggests the best settings depend on cache size.
