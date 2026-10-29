# Cycle 0009 Insight

## Cohort outcome

Two variants tried to stack hit-rate levers. v001 ported the guarded tie-break, with its threshold raised to 3, onto the incumbent double-count line. v002 added the clock-preserving `tick()` to cycle 0008 v002's dedup plus tie-break line. v003 replaced protected move-to-tail with a CLOCK reference bit kept in a separate bitset. All three are Winners by name only. None beat its parent on summary hit-rate. v001 gained in only one large-size cell. v002 lost three cells and gained none. v003 regressed every hit-rate cell and every latency percentile.

## Patterns

- Every bonus for the window candidate fixes the same large-size newcomer loss, so the bonuses overlap instead of adding up. The double count, the tie-break and the restored clock each target that loss. Combining any two adds nothing at the large size and over-favours recency at the small size, where estimates are low, collide often and produce many ties.
- The small-size gain from cycle 0008 v002 comes from slower aging, not from the tie-break. Re-adding the tick took it away. At small capacity, a short aging period is what hurts.
- The idea that get p99 comes from the protected relink is now ruled out. Keeping the bit outside `Node` avoided cycle 0008's load-then-store problem, but the CLOCK policy itself breaks recency order in the protected segment and makes promotion heavier. This is the fourth failed attempt to explain the get tail as a pointer-chase or link-write cost.
- Latency noise from run to run struck for the third cycle in a row. v002 got faster on code that was unchanged or slightly heavier on the hot path. Only v003's consistent regression across all four latency statistics looks like a real effect.

## Open questions

- Does a size-dependent aging rate (slower `SAMPLE_FACTOR` at small capacity) keep cycle 0008 v002's small-size gain while helping the large size?
- Should the tie-break threshold scale with capacity or sketch width instead of being a fixed constant?
- Where does get p99 come from? The inlined sketch update and the map probe are the remaining suspects.

## Continuation

Keep cycle 0008 v002 (dedup, no tick, tie-break `>= 2`) as the hit-rate base. Drop the uniform tick, and drop stacking the tie-break on the double-count line. Probe aging rate, or tie-break thresholds, that depend on capacity. Drop CLOCK or reference-bit replacements for protected LRU entirely. For get p99, profile the sketch update and the map probe instead of the list links, and make no latency claim without interleaved repeat runs.
