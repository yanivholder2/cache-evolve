## Recurring wins

- **Filtering beats protecting.** From 0001 to 0009, the winners made it harder to get into scarce or protected space: a stricter stash→target admission bar (0001), rarer and then no blind BIP MRU admission (0006, 0008), and a recency gate on ghost hits (0007). Variants that added protection, or let weaker evidence in, lost or broke even.
- **Fresh keys face a harsh bar, proven ex-residents are protected (an SLRU/2Q-like split).** This first appeared as MRU stash re-entry for demoted items (0002). It was then applied through where items are placed: fresh keys land at the LRU end (0004), and only evidence-backed keys go to MRU (0005–0008).
- **Where a key is placed is the lever, not its counter.** Pre-credit that only changes a counter did nothing (0004). The same ghost history worked once a ghost hit changed where the key landed (0005).
- **Recency matters more than amount of evidence.** A bigger ghost window lost (0006). A gate at about half a window won (0007). A ghost table halved to fit the gated horizon held hit-rate (0009). Useful ghost history ends somewhere under one cache's worth of evictions.
- **Simplification often wins outright.** Removing BIP (0008) and shrinking the ghost (0009) won while cutting state and branches. Removing a redundant path is worth trying before adding a mechanism.
- **Orthogonal filters stack.** Gating the selective path and removing the blind path compounded (0008). They stacked because they act on different admission routes.

## Recurring failures

- **Stacked retention on the same items.** Scan-skip for credited or ghost-admitted keys (0003, 0006) and a fresh-key doorkeeper (0009) added nothing. MRU placement plus the second-chance scan already protect those items.
- **Taking protected capacity for fresh keys.** Wider probation (0005), set-dueling DIP (0005), MRU landing in the normal tier (0001) and an oversized ghost window (0006) all lost. On this trace the protected stash capacity is close to zero-sum.
- **Counter or frequency gating where position already filters.** A frequency bar on target→probing (0002), a stash threshold of 3 or more on a mature base (0003), and a TinyLFU duel against the LRU-end resident (0004) were all flat. Position and frequency are coupled in this design.
- **Victim choice and routing tweaks.** Cascade-free routing (0003, 0007, the second time bit-identical to its parent), stash victim choice (0003) and spill-to-free-slot (0001) were all flat. Any victim they pick is cheap to recover, and free slots are rare in steady state.
- **Fine-tuning that has plateaued.** Gate-cutoff sweeps, graded mid-stash placement (0008) and shifting the stash/normal slot ratio (0009) moved hit-rate only at noise level. Capacity looks roughly interchangeable between the tiers.
- **Process failure: unmeasured axes.** Efficiency hypotheses (per-op-time, metadata) keep being proposed and keep "winning" on dimensions nobody measured. Writers have never inspected the overlay source, so a no-op cannot be told apart from a real null result.

## Open questions

- Do any of these results hold on a second trace or cache size? Every result so far comes from one LIRS trace at one cache size. The biggest risk is that removing BIP and the short ghost horizon overfit that trace, because they would lose on traces whose useful reuse falls outside the horizon.
- Metadata-size and per-op-time have never been reported. Are the claimed savings real for the halved ghost, BIP removal and cascade-free routing?
- How small can the ghost get before collisions cost hits (a quarter, an eighth)? Would a set-associative ghost push that point further out?
- Does the cascade/displacement path ever fire at this cache size? Did the bit-identical variants (0003 v001, 0007 v003) actually apply their changes?
- Hit-rate levers inside the stash look exhausted. What is left must change who reaches the stash at all, or must come from structural changes measured on new workloads.

## Per-policy notes

- **Current lineage (stash plus normal tier, ghost-guided):** the base is now fresh keys at stash LRU, a second-chance scan, MRU re-entry for demoted ex-residents, MRU admission on recency-gated ghost hits, no BIP, and a ghost about half the cache in size. Hit-rate on this trace has plateaued since about 0008, and further gains are at noise level.
- **Normal tier:** buckets are almost always full, and Front landing already filters the target→probing edge. The neighbour demotion cascade stopped mattering once the stash could recover victims cheaply.
- **Stash:** this is where the real gains have come from. Its value comes from being an evidence-gated victim cache, not from its size or its victim-selection rule.
- **BIP/DIP:** these helped only before the ghost existed. Afterwards they were a net cost, and the best setting is off. Adaptive dueling was noisy and lost.
- **Ghost table:** it helps only when a ghost hit changes placement and the hit is recent. Its size should match the recency gate, not exceed one cache.
