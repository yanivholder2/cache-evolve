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
