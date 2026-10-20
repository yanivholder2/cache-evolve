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
