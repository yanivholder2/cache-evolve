# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three hypotheses started from cycle 0007 v001 (tag-first find, single hash, 7/8 cap), and each changed one separate axis:
- v001 merged cycle 0007 v003's 15/16 cap.
- v002 laid out the bucket hot/cold, with tags inline and ghost/policy state boxed.
- v003 put ghost-miss cold keys at probation LRU instead of mid-probation.

v001 and v003 are Winners on merit. v001 gained hit rate with no latency cost and a better put tail. v003 is the first hit-rate gain on this lineage that leaves the cap alone, and it also made put faster. v002 did not lower get latency and only moved put cost around, so it is a Winner by default.

## Patterns

- The best changes remove work. v003's LRU entry drops a rotate, and v001's smaller probation shortens the cold-insert rotate. Both improved hit rate and the put tail at once. Changes that add indirection to save memory touches (v002's cold box and inline/heap branch) cost as much as they save.
- The cap carried forward cleanly. The merge reproduced the hit-rate target exactly on top of tag-first lookup, which confirms that lookup and replacement are orthogonal. The worry about protected-rotate cost at 15/16 did not show up.
- Get latency no longer depends on struct locality. After tag-first scanning, the working set is likely cache-resident, so get now costs instructions and data movement per hit, not cache lines.
- Admission is now the hit-rate axis with room left. Where a history-less key enters decides whether it or the just-demoted protected item is evicted next. The lever only works when probation has at least 2 slots, though, and the 15/16 cap shrinks probation. v001 and v003 therefore pull against each other: at 16-slot buckets, the merged cap leaves a 1-slot probation, where LRU and middle entry are the same.

## Open questions

- Do v001 and v003 stack, or does 15/16's thinner probation (one slot at 16-slot buckets) cancel LRU insertion there? Would a probation floor of 2 at all sizes ≥16 be a better point once cold keys enter at LRU?
- With one-hit-wonders now evicted first, does the cap even need to be this large? Could a wider probation with LRU entry beat 15/16 on hit rate?
- Would a stronger admission signal, such as a frequency sketch or a ghost-hit count, beat a single ghost bit?
- The miss-path costs are still unmeasured: the linear ghost scan and whole-bucket insert/remove shifts.

## Continuation

Merge v001 and v003: 15/16 cap plus cold keys at probation LRU, on the tag-first base. Then test the probation floor jointly with LRU entry, since the cap and admission now interact through probation width. Keep pursuing admission and ghost policy for hit rate. For latency, target per-hit work and miss-path data movement (non-linear ghost lookup, cheaper probation shifts). Drop bucket-struct layout and hot/cold splitting, and do not add indirection to the put path.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Each variant tested one axis:
- v001 stacked the 15/16 cap on cycle 0008 v003's LRU cold entry, with a probation floor of 2.
- v002 added a per-bucket count-min frequency gate for admission.
- v003 swapped the miss-path whole-bucket shifts for an in-place probation rotate, plus a branch-free ghost scan and an MRU early return in `hit()`.

Only v001 moved hit rate up, by a small amount, and it paid for it in get and put tail latency. v003 kept hit rate the same but was slightly slower. v002 lost hit rate and was slower. v002 and v003 are Winners by default, not on merit.

## Patterns

- The cap and LRU cold entry stack, but only just. The floor of 2 never took effect, because 0008 v003 already kept 2+ probation slots at 16–31. So v001 is really "15/16 on buckets of 32+ slots". The cap axis is close to saturated: making probation narrower now mostly buys longer rotates and more ghost-hit re-inserts on the tail.
- An admission signal is only worth something if it can tell keys apart. The undersized, slowly aged sketch saturated, so nearly every key counted as "seen". Admission quietly fell back to MRU entry, which undid v003's one-hit-wonder protection, and the put path got heavier for nothing.
- The miss-path memmove has now been measured, and it is not a cost. Shifting 16–32 sixteen-byte items on the minority of puts that are full ghost hits is cheap. Removing the shift saved nothing visible. The fixed extras that came with it (a ghost scan with no early exit, an MRU branch that rarely fires under 15/16) made both medians worse.
- Across the cohort, every latency regression came from extra per-op work: branches, counter updates, a larger bucket footprint. None came from bytes moved. Get and put now cost instructions per op, not data movement.

## Open questions

- Would a sketch sized correctly help? That needs enough counters for the window size divided by the threshold, index bits taken from more of the tag, and a saturation check done before measuring.
- Would a ghost-hit count, or a 2-bit ghost entry, give a better admission signal than a sketch, at lower cost?
- Is v001's small hit-rate gain worth its tail cost, compared with 0008 v003's 7/8 cap at 32+?

## Continuation

Use cycle 0009 v001 as the hit-rate base, but treat the cap as finished: change neither the cap nor the probation floor any further. Drop miss-path memmove work, branch-free ghost scans and MRU short-circuits, because data movement on the miss path is not the bottleneck. For hit rate, retry admission only if the signal is checked to discriminate before it is measured (a correctly sized and aged sketch, or ghost-hit counts). For latency, cut fixed per-op instructions on the get-hit path rather than bytes moved.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
