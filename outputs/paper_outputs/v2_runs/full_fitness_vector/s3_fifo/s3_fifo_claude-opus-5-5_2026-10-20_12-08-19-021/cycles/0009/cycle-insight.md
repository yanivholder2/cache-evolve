# Cycle 0009 Insight

## Cohort outcome

All three variants used 0008 v001 (packed `freq` in the index) as their parent:

- **H01** raised `MAX_FREQ` to 7 as a 3-bit packed counter and drained it by halving.
- **H02** replaced hashbrown with a custom linear-probe `(K, u32)` table.
- **H03** sent ghost hits back through small with freq 1 instead of straight to main.

All three are Winners only on rank. Each one regressed something, and none beat the parent cleanly. v001 cut the `put` tail but lost a little hit-rate. v002 kept hit-rate the same and trimmed the `get` tail, but `get` p50, `put` and the struct header all got worse. v003 lost clearly on hit-rate and `put`, which is the falsifying outcome its hypothesis predicted.

## Patterns

- **Hit-rate depends on how many laps a key survives in main, not on how high its counter can go.** Halving bounds the drain cost, but it also removes the long linear retention that produced 0008 v003's gain. Together with 0008, this shows the depth lever is a trade between retention laps and drain work. A cheaper decay schedule does not separate them.
- **Direct ghost-to-main readmission holds the adaptive split together.** Ghost hits shrink small, and the readmitted key fills the slot main just gained. Routing ghost hits through small breaks that link: main grows empty while small starves toward its floor, and long-reuse keys bounce between small and ghost. Every probe of the ghost path has now lost hit-rate, whether it made readmission stronger (0002, 0005, 0008) or weaker (here). The current ghost behaviour looks like a local optimum.
- **Hash-table internals no longer limit `get`.** A leaner probe did not move `get` p50, and linear-probe misses plus backward-shift deletes moved cost onto `put`. The remaining `get` cost is the hash plus two dependent loads (the bucket, then `slots[i].value`). Earlier cycles found inlining the payload slow, so lowering that floor again needs something other than probe tuning.
- **Latency attribution is still unreliable.** v001 and v003 both saw `get` move even though their `get` code was effectively identical to the parent's. No variant had a same-run parent baseline.
- **`small_size`, `settled_small` and `floor_blocks` are still unreported.** This is the ninth cycle in a row. v003's mechanism (small pinned at the floor) is plausible but still unverified.

## Open questions

- Can 0008 v003's retention depth be kept with bounded eviction work, such as a fixed scan budget per eviction or decrementing a sampled subset, instead of a shorter retention schedule?
- Would a prefetch of `slots[i].value` issued from the probed bucket cut `get` p50 on hashbrown, without the costs of the inline layout?
- Is v001's `put`-tail gain real, or drift? Only a same-run re-baseline can tell.

## Continuation

Keep 0008 v001 as the parent, on hashbrown with decrement drain and direct ghost-to-main readmission. Drop halving decay, the custom probe table and all further ghost-readmission changes, in either direction. If the frequency-depth lever is pursued, keep the linear retention and bound the eviction work per `put` instead. Treat `get` as close to its floor unless a variant removes the second dependent load. The loop infrastructure must still run parent and child in the same benchmark and get the split-report stdout to the insight writers.
