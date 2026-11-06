# Cycle 0008 Insight

## Cohort outcome

All three variants built on 0007 v002's stash presence mask and went after its upkeep cost in different ways. v001 used a coarse counting mask, v002 used per-slot fingerprints on the stash bucket, and v003 swapped in a cheap fmix64-style hasher in place of SipHash. v003 is the Winner. It improved every per-op percentile and kept hit rate flat within noise. Both structural fixes repaired `put` but gave back the `get` p50 gain.

## Patterns

- Hashing cost was a first-order term all along. 0007 v001 cut the number of hash calls and p50 did not move. v003 made each call cheaper and p50 dropped. Making each re-hash cheaper also shrank most of the mask-refresh penalty on `put` and on the `get` tail, without changing the refresh structure.
- Mask upkeep affects only `put`. Both O(1)-maintenance variants (v001, v002) fully recovered `put`. Neither recovered `get` p99, and neither kept p50. Making the update cheap does nothing for reads.
- The read fast path depends on a dense, separate mask word with low false positives. v001 shrank the filter, false positives rose, and the filter ended up costing more than no filter at all. v002 moved presence tracking onto the bucket's own line as a byte scan, which was slower than one AND against a compact mask word even though it was more exact. The "extra cache line" that 0007 suspected turned out to be cheap for reads.
- The `get` tail sits on the stash-hit/promotion path, not in the mask refresh. v002 removed the refresh and the tail still got worse, because adding work to the item shift on promotion lands directly on that path.

## Open questions

- With cheap hashing in place, does a structural O(1) mask update still pay off? Any such update must keep the dense 64-bit (or wider) word so the false-positive rate does not rise.
- Is there more p50 left in the common-path hash, for example the segment-level selection, or in the remaining false positives on the mask test?
- Would the cheap hasher also change the trade-offs on the deferral line (0005 v002 lineage), since that line's costs were partly re-hash-driven?

## Continuation

Build on 0008 v003 (fast hasher plus the 0007 v002 mask). Keep the separate dense mask word for normal-hit rejection. Drop shrunken counting masks and in-bucket fingerprint scans as the reject path. If `put` needs to go lower, try O(1) mask maintenance that keeps at least 64-slot resolution. Also look at the cost of the stash-promotion shift for the `get` tail, without adding work to that shift.
