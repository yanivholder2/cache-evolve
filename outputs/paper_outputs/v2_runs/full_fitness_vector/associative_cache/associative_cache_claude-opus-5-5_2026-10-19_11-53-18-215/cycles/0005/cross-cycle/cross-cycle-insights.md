# Cross-Cycle Insights (through cycle 0005)

## Recurring wins

- **One hashed bucket per key, with one shared policy over the merged bucket.** Since cycle 0001, every Winner has kept each key in its single hashed bucket. Starting in cycle 0002, every Winner has also used one policy across the whole merged bucket.
- **Cutting data movement is the most reliable latency lever (cycles 0002–0005).** Cost is dominated by memmove, so these all helped:
  - narrower `Item` stride: 40 → 24 → 16 bytes
  - shorter rotates
  - eviction confined to probation
  - returning the insert index instead of re-finding
  
  None of them changed hit rate. The largest gains showed up at the tails, where the longest shifts happen.
- **Shrinking metadata is free under position-ordered policies.** Timestamp and LFU fields were never read under LRU/SLRU. Replacing wall-clock reads with logical ticks, then removing the fields entirely, improved metadata-size and put with hit rate flat.
- **Giving protected more room raises hit rate.** This held across cycles: scan resistance (SLRU) beat split-half and plain designs, and raising the fixed protected fraction from 3/4 to 7/8 gained more.
- **Ghost history helps only when it decides where a new key enters probation** (cycles 0004–0005). In that role, cold keys enter mid-probation and ghost hits enter at MRU.
- **Changes to separate axes stack cleanly.** Hit rate (protected capacity, placement), get median (tag-first scan), and metadata/tails (stride) have each moved on their own. Merging Winners from different axes has worked every cycle.

## Recurring failures

- **Placing a key outside its hashed bucket** (two-choice, cuckoo, cross-bucket victim areas). These always fail the P2 slot_legality invariant.
- **Split home/partner LRU halves.** The bookkeeping cancelled the extra flexibility and hit rate fell below baseline.
- **Anything that shrinks or displaces protected.** Two attempts failed:
  - ghost-to-protected admission (cycle 0003) demoted items that had earned their place with a real second hit
  - ARC-style adaptive protected sizing (cycle 0004) drifted toward LRU, because its grow signal rounds to zero in small buckets
  
  Both lost scan resistance on this LIRS-like trace.
- **Lookup order that doesn't match the segment layout.** Scanning from the MRU end forces every probation hit to walk all of protected first, which hurt the get median. Earlier credit given to "MRU-first" scanning actually belonged to the shorter moves.
- **Ghost machinery has put-tail costs:** a second hash, a linear ring scan on every miss, and ring writes on every eviction. Its per-bucket memory is also invisible to the metadata-size dimension.
- **Fragile carry-forward.** Merges have silently dropped earlier wins (for example, a shrunken `item.rs`), which then shows up as false regressions.
- **Latency shifts with no code cause.** These have recurred (cycles 0003, 0004, 0005). Small per-op shifts are not trustworthy without a side-by-side re-run.

## Open questions

- On the 16-byte `Item`, does keeping the tag array in sync on put cost more than the single hash saves?
- Can ring offsets or swap-based promotion remove the moves that span all of protected (protected-hit rotates, probation→protected promotion, ghost-hit / `len0 == 0` shifts)? These moves get longer as protected grows.
- Has hit rate peaked at 7/8? Would 15/16 help large buckets? How does the probation floor interact with bucket size and with mid-probation insertion?
- Can the ghost fingerprint come from the existing hash, and can ring lookup avoid a linear scan? Would a larger ring catch longer reuse distances? Is mid-probation the best entry point for cold keys?
- Plain LRU vs SLRU on the same base is still untested after five cycles.
- Was cycle 0005 v002's slowdown real, or noise?

## Per-policy notes

- **Split-half LRU (home/partner):** fails on hit rate. Its early speedup came from how it was built (one hash per op, contiguous bucket storage, in-place policy updates), not from the policy.
- **Plain LRU over the merged bucket:** recovered hit rate above baseline. It has never been compared directly with SLRU.
- **SLRU (probation/protected, boundary stored in `len0`):** the current lineage. Scan resistance with no per-item metadata. Its cost is movement, not the policy. Keep evictions inside probation and keep protected large (7/8 currently best).
- **SLRU + ghost ring:** placement-only use gains hit rate. Using it for protected admission or protected sizing loses hit rate. It costs put tail unless made cheaper.
- **ARC-style adaptive split:** loses on this trace and bucket geometry. Drop it.
- **Timestamp/LFU-based policies:** not pursued. Their fields have been removed from `Item`, so reviving them would cost metadata-size.
