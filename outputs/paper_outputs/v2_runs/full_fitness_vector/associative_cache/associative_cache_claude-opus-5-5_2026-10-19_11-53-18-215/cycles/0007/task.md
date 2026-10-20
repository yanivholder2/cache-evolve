# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Two variants built on cycle 0003's v001 (24-byte SLRU), and one on its v003 (ghost ring):
- v001 combined the movement fixes and returned the insert index instead of re-finding.
- v003 used ghost history to choose where a new key enters probation: cold keys go to the middle, ghost hits go to MRU.
- v002 replaced ghost-to-protected admission with an ARC-style adaptive protected cap.

v001 and v003 are the Winners:
- v001 made put faster. Its get median got slower.
- v003 gained some hit rate and made the median put faster, but its put tail got worse.

v002 lost hit rate.

## Patterns

- Put-side movement fixes now clearly stack. Evicting only from probation, keeping each move short, and returning the insert index all made put faster with SLRU semantics unchanged. Both Winners used the same short-rotate eviction, and both got the put improvement from it.
- How lookups are ordered has to match the segment layout. Scanning from the MRU end makes every probation hit scan all of protected first. That hit the get median, which is the summary for per-op time. Cycle 0003 attributed get gains to "MRU-first". That credit belonged to the shorter moves, not to the scan direction.
- Ghost history gives hit rate when it only decides where a key is placed. It loses hit rate when it decides how big protected is. v003 never displaces protected items and gained hit rate. v002 drifted toward plain LRU and lost scan resistance. It had a frequent shrink signal (ghost hits), and its grow signal rounds to zero slots in small buckets. Cycle 0003's ghost-to-protected attempt also failed. The lesson from both: anything that weakens protected hurts on this trace.
- Ghost machinery costs time on the put tail. It needs a second hash for the fingerprint, scans the ring linearly on every miss, and writes to the ring on every eviction. Its per-bucket memory does not show up in the metadata-size dimension.
- The cohort again saw latency shifts that the code does not explain. v002's slowdown may partly be cycle 0003 v003's unexplained gain going away.

## Open questions

- Would a segment-aware lookup bring back the get median while keeping v001's put gains? One order is probation MRU→LRU first, then protected. The other is a plain forward scan.
- Can the ghost fingerprint come from the existing hash (no second hash), and can the ring be checked without a linear scan? Either would shrink v003's put tail.
- Is the middle of probation the best entry point for cold keys? Would a larger ghost ring catch longer reuse distances?
- Plain LRU vs SLRU, and a fixed protected fraction other than 3/4, are still untested after three cycles.

## Continuation

Merge the two Winners. Start from v001's put path (probation-only rotate, returned index). Switch `find` back to a forward scan or a segment-aware scan. Add v003's mid-probation cold insertion. Then make v003's ghost path cheaper: derive the fingerprint from the existing hash, and give the ghost-hit and `len0 == 0` branches the short-rotate eviction. Drop adaptive protected sizing and any other mechanism that shrinks or displaces protected. If any hit-rate work is left, spend it on testing fixed protected fractions.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
