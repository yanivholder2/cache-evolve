# Cycle 0010 Insight

## Cohort outcome

Two variants worked on hit-rate and one on latency. v001 restored cycle 0008 v002 (dedup, no tick) and slowed aging only at small capacity, with `SAMPLE_FACTOR` 16 at ≤ 32768. v002 picked the colder of the first two probation entries as the victim. v003 replaced the hashbrown index with a packed linear-probing table. v001 and v003 are real Winners. v001 gained in three hit-rate cells and lost none. v003 cut get p50/p99 and put p50 with hit-rate bit-identical, at the cost of a worse put p99. v002 is a Winner by name only: it lost in three cells, including the large-size cell it targeted.

## Patterns

- Aging rate is confirmed as the small-size lever, and it has not saturated yet. Gating it by capacity keeps the large size isolated, so the rule that a safe lever touches only one size holds. Dropping the tick also gave the large size a small gain, so dedup-without-tick is now the right base at both sizes.
- Victim-side tweaks run into the same problem as candidate-side bonuses. Picking a colder victim lowers the admission bar, so it stacks with the tie-break and loosens the filter. Under LRU-ordered probation, "colder by estimate" mostly means "younger and not yet re-referenced", so the change punishes recency. That is the opposite of the stale-entry signal it was meant to find.
- After four failed tries at explaining the get tail, the map probe is the first one that held. v003's gain is spread across get p50, get p99 and put p50, and it matches the change's mechanism, so unlike past swings it is unlikely to be noise. Its put-p99 cost also has a plain cause: backward-shift deletion plus a separate insert probe on the eviction path.
- Latency noise from run to run struck for the fourth cycle in a row. v001 got slower on code that is identical or lighter, and v002's get tail got worse with `get` unchanged. Hit-rate and latency verdicts still need to be read separately.

## Open questions

- Where is the ceiling on the small-size aging factor? 4-bit counter saturation is the expected limit.
- Does a moderately slower aging factor help the large size too, now that dropping the tick there gave a small gain?
- Can v003's put-p99 cost be removed by merging the `remove(old)` and `insert(new)` probes, or by a lower load factor, while keeping the get gain?
- Does v003's latency gain hold up in interleaved repeat runs, and does it add to v001's change once both are on one line?

## Continuation

Merge v001's capacity-gated aging with v003's packed index into one base. The two changes are independent: v001 changes the policy and v003 changes only lookup. Next, sweep the small-size factor upward to find where saturation sets in, and try a mild large-size slowdown. On the index, go after the eviction-path put p99 (fused remove+insert, or a lower load) without giving back get latency. Drop victim selection by raw estimate comparison. Revisit the victim side only with an age-aware or stale-frequency signal. Confirm latency claims with interleaved runs.
