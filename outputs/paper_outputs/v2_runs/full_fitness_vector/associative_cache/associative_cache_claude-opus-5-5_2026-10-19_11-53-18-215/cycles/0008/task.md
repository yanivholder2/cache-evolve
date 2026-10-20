# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three hypotheses started from cycle 0004's v003 (ghost-guided mid-probation insertion), and each changed one separate axis:
- v001 added a parallel 16-bit tag array, scanned forward before any full-key compare, and hashed once instead of twice.
- v002 raised the fixed protected fraction from 3/4 to 7/8, with a 2-slot probation floor.
- v003 cut `Item` down to `{ key, value }`.

All three are Winners. v001 improved the get median. v002 gained hit rate. v003 brought metadata-size to its floor and made get and put faster, mostly at the tails.

## Patterns

- Each axis now has a lever that works on its own: tag scanning for the get median, protected capacity for hit rate, and stride width for metadata and tails. Because each variant changed a different axis with the others held fixed, they should combine with little interference.
- The 16-byte `Item` confirms that this design's cost is mostly memmove. A narrower stride helped every path with hit rate exactly unchanged, and the biggest gains came at the tails, where the longest shifts happen. The tag array goes after the other cost, the scan width on misses.
- v002 strengthens the cross-cycle rule that giving protected more room helps on this trace. The fixed 3/4 split had not reached its limit. The gain depends on bucket geometry: the 2-slot floor cancels 7/8 at 8 slots and shrinks protected at 4 slots.
- Carry-forward is fragile. v001's metadata-size and put regressions look like a lost carry-forward of the shrunken `item.rs`, not a cost of tagging. Its put cost cannot be judged until it runs on the 16-byte `Item`.
- Latency again moved without a code cause. v002 changed only construction-time arithmetic, yet get and put both got slower. The cause is either noise or longer protected-side rotates.

## Open questions

- On a 16-byte `Item`, does keeping the parallel tag array in sync (insert, rotate, remove) cost more on put than removing the second hash saves?
- Is v002's slowdown real? A larger protected segment makes protected-hit rotates and the ghost-hit/`len0 == 0` shifts longer. Re-run it side by side with its parent.
- Has hit rate peaked at 7/8, or would 15/16 help on large buckets? Does a probation floor of 2 starve mid-probation insertion?
- Plain LRU vs SLRU is still untested after four cycles.

## Continuation

Merge all three Winners into one base: the 16-byte `{ key, value }` `Item`, tag-first forward `find` with a single hash, and a 7/8 protected cap. Then measure the put cost of tag-array upkeep on that base. Next, try cheaper protected-side moves (ring offsets or swap-based promotion), since a larger protected segment makes those moves longer. If hit-rate work remains, probe protected fractions above 7/8 and check bucket-size sensitivity. Keep dropping anything that shrinks or displaces protected.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
