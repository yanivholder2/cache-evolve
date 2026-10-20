# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three hypotheses started from cycle 0005's v003 (16-byte `{ key, value }` SLRU), and each changed one separate axis:
- v001 put the 7/8 protected cap, with a probation floor that scales with bucket size, onto the 16-byte item.
- v002 skipped the rotate on protected hits in the MRU quarter (lazy promotion).
- v003 doubled the ghost ring and removed one of the two extra hashes on put.

Only v001 actually moved anything: hit rate went up as predicted and metadata stayed at its floor, but get and put both got slower. v002 was neutral on every dimension. v003 was flat-to-worse on hit rate and slower on put. All three are listed as Winners, but v002 and v003 are Winners by default, not on merit.

## Patterns

- The protected cap now trades hit rate directly against latency. Cycle 0005 v002's slowdown was real: it showed up again on the narrower stride. A larger protected segment makes protected-hit rotates longer, and a 1-slot probation makes evictions and ghost re-inserts (whole-bucket shifts) more frequent. "More protected helps" still holds for hit rate, but it is no longer free.
- Lazy promotion near MRU saves nothing. Hot keys already sit at or next to the MRU slot, where a rotate moves almost nothing. The memmove cost lives in hits deep in protected and in probation promotions, and skipping those moves is exactly what would cost hit rate. Any latency fix has to make deep moves cheaper, not skip shallow ones.
- Ghost-ring length is exhausted. A ring of half a bucket already captures the reuse that placement can exploit. Longer windows add false positives, push fresher probation items toward eviction, and lengthen the linear ring scan that every miss-path put pays. Removing a single hash is too small to offset an O(ring) scan.
- Cycle 0005's tag-first `find` was never carried into this cohort. All three variants branched from the untagged parent, so the get-median lever from 0005 is still sitting unmerged.

## Open questions

- Can ring offsets or swap-based promotion make deep protected moves cheap enough to keep 7/8's hit rate without its latency cost?
- Is a cap between 3/4 and 7/8 (or a 2-slot probation at 8 slots) a better point on the hit-rate/latency curve?
- Would a non-linear ghost lookup (tiny bitset or hashed slot) shrink the miss-path put tail at the current ring size?
- What does tag-array upkeep cost on put once it runs on the 16-byte item?

## Continuation

Build the base from cycle 0006 v001 (16-byte item, 7/8 cap) plus cycle 0005 v001's tag-first single-hash `find`, carrying over both `item.rs` and the tag array explicitly. Spend latency work on cheapening deep protected and probation moves (ring offsets, swap-based promotion) and on replacing the linear ghost scan, not on skipping near-MRU rotates. Drop ghost-ring enlargement and lazy hot-zone promotion. If you probe the cap again, treat it as a hit-rate/latency trade-off, not a pure win.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
