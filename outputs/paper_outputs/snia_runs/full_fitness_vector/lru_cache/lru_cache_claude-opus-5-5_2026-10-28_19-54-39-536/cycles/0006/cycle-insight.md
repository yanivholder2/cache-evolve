# Cycle 0006 Insight

## Cohort outcome

Two variants branched from cycle-0005 v001 (the size-gated `prev`-bit reprieve). v001 cut the ghost's hashing cost (Fx tags, vacant-path only, single-OR `set_protected`). v003 turned the reprieve back on at 131K with a much smaller ghost, sized at capacity/8. v002 branched from cycle-0004 v002 and replaced hashbrown with a custom tombstone-free index table. All three are Winners, but each won only partly. v001 got most of the median put cost back with no hit-rate loss. v002 fixed the get tail but lost get p50. v003 was roughly neutral on hit rate at 131K.

## Patterns

- Two get-p50 explanations have now been ruled out. v001 removed the flag arithmetic from `on_hit` and get p50 still regressed. v002 touched fewer cache lines per lookup and still lost to hashbrown's SIMD group probing. So get p50 does not depend on flag bit-ops, list work or lookup cache lines. The leading suspects left are code layout and inlining of the shared helpers, and the ghost table's cache footprint.
- Put cost tracks hash count, now for the fourth cycle. A cheap fingerprint hash recovered the put median, and a 16-bit tag does not need a strong hash. v002's backward shift, which runs a full SipHash for each entry it moves, pushed the put tail up the same way.
- Removing the victim before inserting the new key fixes the get tail. v002 is the first evidence that the cycle-4 get-p99 rise was real and came from tombstone churn, not noise.
- Noise now hides the effects this cycle measured. v003's timings swung sharply on code that was byte-identical to v001. Its 16K hit rate moved on unchanged code, most likely because `RandomState` is reseeded on every run. Small hit-rate deltas cannot be trusted while the ghost hasher is unseeded or randomly seeded.
- An undersized ghost works as a reuse-distance filter, but at capacity/8 it is too short for web_0 at 131K. It forgets most of the useful re-references before they come back.

## Open questions

- Does a wider large-size ghost (capacity/4 or capacity/2) recover cycle 4's web_0 gain at 131K while still keeping out hm_0's long-loop keys?
- Is the remaining get-p50 regression caused by inlining and layout of `on_hit`/`unlink`/`push_back` after the put restructure? Forcing or blocking inlining of those helpers would test this.
- Does remove-before-insert on the stock HashMap parent get v002's tail fix without losing hashbrown's probing speed?

## Continuation

Use cycle-0006 v001 (Fx ghost tags, vacant-only ghost work) as the base. Fix the ghost hasher's seed first, so hit-rate deltas can be measured, and time parent and child in repeated paired runs. Then try remove-before-insert eviction on hashbrown, and a large-size ghost wider than capacity/8. Drop the custom scalar index table, and drop further bit-op trimming in `on_hit`. Look into the get-p50 regression through helper inlining and code layout.
