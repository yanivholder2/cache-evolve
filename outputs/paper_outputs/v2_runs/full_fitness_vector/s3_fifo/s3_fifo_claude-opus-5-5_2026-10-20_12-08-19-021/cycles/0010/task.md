# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle had one layout change and two single-lever policy probes:

- **H01** packed the 2-bit `freq` into spare high bits of the index value. Its parent was 0007 v001.
- **H02** grew the ghost to the full `capacity`. Its parent was 0006 v002.
- **H03** raised `MAX_FREQ` from 3 to 7. Its parent was 0006 v002.

All three are Winners. v001 is the only one that improved without regressing anything. `get` got faster, it keeps the 8-byte `Entry` with no per-slot cost for `freq`, and hit-rate is unchanged. v003 gained a little hit-rate but paid for it in `put` and `get` latency. v002 lost on both hit-rate and `put` tail latency and made the cut on rank.

## Patterns

- **Bit-packing resolves the conflict between hot-field splitting and latency.** In 0007, moving `freq` into a separate array cost `get` a second memory access. Putting it in the index bucket that `get` already probes gives the metadata saving with no extra access. The rule now holds across three cycles: the `get` path should touch only memory it already reads, which is the probed bucket plus the value.
- **Making the ghost path stronger has backfired again.** A larger ghost readmitted keys whose reuse distance is too long to survive in main, and those keys pushed out better residents. Earlier failures from the ghost path were ghosting main victims (0002) and readmit-with-`freq` (0005). This is the third, so the evidence now suggests ghost capacity is not what limits hit-rate.
- **Retention depth is a real hit-rate lever, but an expensive one.** `MAX_FREQ` is the first lever since the floor to move hit-rate in the right direction. The cost lands entirely on eviction work: longer CLOCK drains slow `put` and, through cache pressure in the mixed stream, also slow `get`. Hit-rate gains on workload-a keep coming in small amounts, and only from the bounds or the depth of the policy, never from trigger rates.
- **Eviction-path costs reach `get`.** v003's `get` code was identical to its parent's, yet `get` regressed. In a mixed benchmark, `put`-side sweep work counts against the summary `get` metric.
- **`small_size` and the split reports are still unreported.** This is the eighth cycle in a row. No writer could read any `h0x_split_report` output, so `floor_blocks` and the rotations-per-eviction figures stayed unverified. v001's `put` gain also still lacks a same-run re-baseline.

## Open questions

- Can the `MAX_FREQ` hit-rate gain be kept with a cheaper drain? Options are halving on each pass instead of decrementing, or a cap of 4–5. A cap of 7 needs 3 bits, which no longer fits v001's 2-bit packing. Taking a 29-bit slot id would make room.
- Does a larger ghost pin small at the floor? Checking `floor_blocks` would tell.
- Is v001's `put` improvement real, or the 0007 drift reversing?

## Continuation

Adopt v001 (packed `freq` in the index) as the parent for both latency and metadata. Drop ghost-capacity growth and any change that strengthens readmission; if the ghost is probed again, try it smaller or make readmission stricter. Pursue the frequency-cap lever only together with a cheaper main drain, such as a halving decay or a cap of 4–5. Packing it as 3 bits into the v001 layout is fine. The loop infrastructure still has to benchmark each parent and child in the same run and get the split-report stdout to the insight writers.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
