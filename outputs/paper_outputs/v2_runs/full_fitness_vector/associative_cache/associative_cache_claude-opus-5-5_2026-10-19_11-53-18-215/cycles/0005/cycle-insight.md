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
