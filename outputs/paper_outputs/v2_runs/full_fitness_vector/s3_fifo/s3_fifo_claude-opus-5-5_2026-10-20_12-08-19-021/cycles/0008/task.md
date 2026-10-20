# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants used cycle 0004's v003 as their parent. Each changed one lever:

- **H01** lowered the small-queue floor clamp from `capacity/16` to `capacity/64`.
- **H02** readmitted ghost-hit keys into main with `freq = 1`.
- **H03** moved values into the hash map to get a single-probe `get`.

All three are Winners, but each won something different. v003 ranked first on `metadata-size`, the first real drop in that metric so far, but `get` got slower. v001 is the only variant that improved hit-rate, and only slightly. v002 lost on both hit-rate and latency and made the cut on rank.

## Patterns

- **The lower clamp binds, but only weakly.** Hit-rate moved when the floor was loosened, so the split does reach `capacity/16` on workload-a. The gain was small, which means either the split only sits at the floor part of the time or the benefit of a bigger main levels off quickly. This is the first cycle where the adaptation's bounds turned out to be a live lever. Movement-rule tweaks in earlier cycles did not.
- **Extra credit on readmission hurts.** Admitting a key straight to main from the ghost is already its reward. Adding `freq` on top kept one-time-reuse keys in main for an extra lap, pushed out keys whose reuse was seen through real `get`s, and added rotation work to `evict_main`. Together with cycle 0002's ghosted-main-victims loss, this suggests that changes making the ghost path stronger tend to backfire.
- **The `get` cost is the probe's memory footprint, not the dependent load.** Putting the payload inline widened the hash buckets to a size that is not a power of two, and `get` got slower at both p50 and p99. That is the first `get` move in several cycles that noise does not easily explain. Counting dependent loads predicted the wrong direction.
- **A slimmer `Entry` and a fast `get` pull against each other only when the entry is stored inline in the probe table.** Shrinking the entry inside the slab could reduce `metadata-size` without widening the index.
- **`small_size` is still unreported.** v001 and v002 both added the instrumentation, but neither writer could see its output. So we still cannot tell whether v002's loss came from a split shift or from main residency alone.

## Open questions

- What does `h01_split_report` show: where does `small_size` settle, and how often did the old floor block a shrink? This needs to surface in the variant insights, not just the variant notes.
- Is the grow-small trigger, which pulls the split back up, now the stronger lever than the floor?
- Is v001's `get` regression noise? Its `get` path is byte-identical to the parent's.
- Can the slab keep a key-free 16-byte `Entry` while the index stays at the parent's narrow `(K, u32)` buckets?
- Would conditional readmit credit (only on a second ghost hit, or only while small is at its floor) behave differently from the blanket version?

## Continuation

Use v001 (the `capacity/64` floor) as the policy parent, since it is the only hit-rate gain. Don't lower the floor any further. Probe the grow-small trigger strength or the upper clamp next, one lever at a time with ghost size pinned. Drop blanket readmit-`freq` credit and inline-payload index layouts. To shrink `metadata-size`, try a key-free `Entry` that stays in the slab behind the narrow index. Each variant must get its settled `small_size` figure into its insight, and must not credit `get` changes unless the `get` path or the index layout changed.


### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants used 0006 v002 as their parent. Two were policy-neutral trims and one changed policy:

- **H01** moved `freq` into a parallel `freqs` array, which shrinks `Entry` to 8 bytes.
- **H02** doubled the ghost-hit shrink step, with the ghost pinned.
- **H03** skipped the `freq` store on `get` when the counter is already saturated.

All three are Winners, but none improved without a regression. v001 ranks first on a real per-slot `metadata-size` drop. v003 and v002 made the cut on rank: hit-rate was flat across the whole cohort, and every variant regressed on latency.

## Patterns

- **The benchmark baseline drifted this cycle.** `put` slowed in all three variants, including v003, whose `put` path is byte-identical to the parent's. So this cycle's latency deltas can't be credited to the code, in either direction. That includes v001's `get` loss, even though it has a plausible cause: a second cache line on every hit.
- **Splitting hot fields from each other is a different trade from splitting cold fields out.** Moving the key out (0006) helped `get`. Moving `freq` out splits two fields that every hit reads together. The metadata saving is real, but at this capacity the slab already fits in cache, so there was little locality to win back.
- **Neither trigger ratio is a lever on workload-a.** Cycle 0006 found the grow-small side inert, and the ghost-hit shrink side is now inert too. If the split responds to anything, it is the bounds (the `capacity/64` floor) or the ghost sizing, not how fast it moves between them.
- **Store-elision micro-trims on `get` are below what this benchmark can resolve.** At this measurement noise, a single-threaded L1-resident store costs too little to show up.
- **`small_size` is still unreported.** This is the seventh cycle in a row. All three writers said the split-report output was not in the artifacts they could reach. The variants did generate it, but it never reaches the insight writer.

## Open questions

- Is v001's `get` regression real, or drift? Only a same-run parent-vs-child benchmark can tell.
- Can `freq` be packed into spare bits next to the value or the index entry? That would keep the 8-byte saving without a second access per hit.
- Does `settled_small` sit at the floor under both shrink steps (`floor_blocks`)? If so, the floor is the binding limit.
- Is the pinned ghost capacity, rather than the split bounds, what caps hit-rate now?

## Continuation

Keep 0006 v002 as the latency parent, and carry v001's `freqs` split only as a metadata-size branch until a same-run re-baseline clears its `get` cost. Prefer a bit-packed `freq` over a separate array. Drop trigger-step tuning on both sides, and drop `get` store-elision. The next policy lever should be the ghost capacity (held relative to the initial main size) or a test of whether the floor binds. Before trusting any latency claim, the loop must (1) benchmark parent and child back-to-back in the same run and (2) copy the `h0x_split_report` stdout into the eval artifacts the insight writers can read.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
