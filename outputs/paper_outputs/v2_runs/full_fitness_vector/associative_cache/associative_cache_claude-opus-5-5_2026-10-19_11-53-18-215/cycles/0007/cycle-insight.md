# Cycle 0007 Insight

## Cohort outcome

All three hypotheses started from cycle 0006 v001 (16-byte item, 7/8 cap), and each changed one separate axis:
- v001 finally merged cycle 0005's tag-first single-hash `find`.
- v002 limited protected-hit promotion to move-ahead-4.
- v003 raised the cap to 15/16 on buckets of 16+ slots.

v001 is the clear Winner: get and put both got faster, and hit rate and metadata did not change. v003 gained a small amount of hit rate at no latency cost. v002 kept hit rate but got slower on get and much slower on put. It is a Winner by default, not on merit.

## Patterns

- Lookup and replacement can be tuned separately. Tag-first lookup with one hash cut latency without changing any replacement decision. Put gained more than expected, because the ghost and eviction paths now reuse stored tags instead of rehashing. Keeping the tag array in sync costs much less than the key compares and hashes it removes. That answers the open question about tag upkeep.
- Long protected rotates are not what makes the 7/8 cap slow. Protected segments are at most 14–15 items of 16 bytes each, so even a full rotate is cheap. Bounding it gained nothing and added variable-length moves. Both v002 and v003 point the other way: the cost comes from the thin probation segment. A 1-slot probation means more evictions, ghost writes and ghost scans, and more whole-bucket insert/remove shifts on put.
- The protected-cap lever is nearly used up. 15/16 still adds hit rate on large buckets, but much less than the 3/4 → 7/8 step did. Its cost was low only because it reuses the existing in-place replace path. Probation now holds cold keys for only one miss.
- Approximate recency inside protected hurt indirectly. It demoted still-warm keys into probation, and that churn showed up on the put tail even though put's own code got lighter.

## Open questions

- With probation down to one slot, how much of the median put cost on a miss comes from the linear ghost scan and ghost writes? Would a hashed or bitset ghost lookup remove it?
- Can admission decide more of the hit rate now that the cap is flattening? Examples: a frequency or ghost-based filter that keeps one-hit-wonders out of probation altogether.
- Do whole-bucket insert/remove shifts in probation eviction dominate the put tail? Would a ring-offset layout for probation alone remove them?

## Continuation

Build the base from v001 (tag-first find, single hash, tags reused on ghost and evict) plus v003's 15/16 cap. Spend latency work on the miss and eviction path: non-linear ghost lookup, and cheaper probation insert/evict shifts. Leave protected-side rotates alone. Look for further hit-rate gains in admission or ghost policy, not in a larger protected share. Drop bounded or approximate promotion inside protected, and stop pushing the cap past 15/16.
