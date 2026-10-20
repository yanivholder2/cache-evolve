# Cycle 0006 Insight

## Cohort outcome

This cycle combined the two cycle-0005 incumbents and probed one new policy lever. H01 put the `capacity/64` floor onto 0005 v003's inline, slab-free layout. H02 moved the key out of the slab `Entry` into a parallel `keys` array behind the narrow `(K, u32)` index, on top of 0005 v001. H03 halved the grow-small trigger with `GROW_SMALL_EVERY = 2`. All three are Winners. v002 is the only one that improved without regressing anything: `get` got faster, `Entry` reached 16 bytes, and hit-rate was identical. v001 matched the floor's hit-rate at minimum metadata-size, but its latency regressed. v003 was neutral on hit-rate and made the cut on rank.

## Patterns

- **Policy and layout are independent levers.** The floor's hit-rate gain reproduced exactly on a different storage layout (v001), and the layout refactor kept the policy exactly the same (v002). So winning levers can be stacked freely across lineages.
- **Splitting hot and cold data is the layout lever that works.** Inlining the payload in the probe table slowed `get` (cycles 0005 and 0006, v001). Moving cold fields such as the key out of the array `get` reads, while keeping the index narrow, sped `get` up (v002). The `get` path should touch only `{value, freq}`.
- **`metadata-size` now overstates savings.** It measures only `size_of::<Entry>`. In v002, the `Option<K>` key array gives back roughly the bytes the 16-byte `Entry` saved, so the real per-slot footprint barely moved. Per-cache fields such as v003's padded `u8` show up in a separate row. Treat this metric as a proxy for the hot array, not for total memory.
- **The grow-small side of the split is not a lever on workload-a.** Damping the grow-small trigger did nothing to hit-rate. Combined with the weak but real gain from the floor, this suggests the ghost-hit shrink side sets the balance point. The "bigger main helps" theory has now run out of cheap triggers to pull.
- **Latency noise is still large, especially at p99.** v001 and v003 both saw `get` move in opposite directions with byte-identical `get` code. Only v002's `get` change has a code-path explanation, and even part of it may be the parent's tail settling back.
- **`small_size` is still unreported.** v003 refers to its `h03_split_report` but gives no settled value. Six cycles have now asked for it.

## Open questions

- Does the settled split actually move under the /64 floor or the damped grow-small trigger? `settled_small`, `grow_events` and `grows` need to appear in an insight.
- Would a niche-free `keys: Vec<K>`, with liveness tracked through `free`, really shrink the per-slot footprint? And would it keep v002's `get` gain?
- Is v001's `put` regression a real CLOCK-walk cost from a longer `main`, or noise? That needs a same-run re-baseline against its parent.
- Does v002's delayed `Drop` of stale values matter for non-trivial `V`?

## Continuation

Adopt v002 as the parent. It has the floor policy, the narrow index and split hot/cold arrays, and it wins on `get` with no regressions. Drop the inline-payload layout (v001's lineage) and the grow-small damping lever. Next, try a niche-free `keys` array to cut the real per-slot footprint, and a same-run latency re-baseline. Only test further policy levers on the ghost-hit shrink side, one at a time with the ghost pinned. Every variant must put its settled `small_size` figure in its insight.
