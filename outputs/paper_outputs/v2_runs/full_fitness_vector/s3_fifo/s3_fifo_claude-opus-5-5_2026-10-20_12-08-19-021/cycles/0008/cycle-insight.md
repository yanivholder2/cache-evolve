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
