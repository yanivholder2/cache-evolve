# Cross-cycle insights

## Recurring wins

- Id-keyed indexing in place of linear resident scans (Cycles 0005–0008). It is the most consistently pursued direction, leaves eviction, ghost, and promotion logic untouched, and shows no hit-rate change. Get and put speedups are plausible but still need a baseline rerun before credit.
- Merging per-queue indexes into one id-keyed map (Cycle 0006) reduced container state and put cost with no hit-rate change. This is the cleanest structural gain in the series.
- Fingerprint-keyed ghost storage (Cycles 0003, 0004, 0005) gave a put-side benefit with no hit-rate change in each cycle it was tested. Its cost is container-level bookkeeping, not per-entry size.
- Whole-structure footprint is the right unit of measurement. Every Winner changed the container; none produced a measurable saving in `Entry` alone.
- Batched, off-path ghost expiry (Cycle 0007) is the first edit to address ghost drain cost directly. Its hit-rate gain is small and rests on one trace.

## Recurring failures

- Lowering the frequency ceiling below 3 (Cycle 0008) cost hit rate on the workload tested and saved no metadata. The third level appears to earn its keep.
- Enlarging the ghost toward full cache capacity (Cycle 0004) admitted more keys to main without a matching reuse gain and lost hit rate. Ghost size is not a free dial.
- Moving singly-reused keys into main earlier (promotion threshold `freq > 0`, Cycle 0001) gave no attributable gain and cost hit rate.
- Stacking a lowest-frequency victim choice on the larger small queue (Cycle 0003) lost hit rate. The victim change has little to act on once the small queue is larger.
- Splitting the frequency counter into a side array (Cycle 0004) grew the container, a net footprint loss. Shrinking `Entry` alone did not shrink the structure, and it added a get-path read per hit.
- Swapping the index hasher for a multiply-shift hash (Cycle 0008) shrank the container but made gets slower. Hashing does not appear to be the lever on the hit path.
- Tagging ghost entries with sequence numbers (Cycle 0002) added put-path bookkeeping with no policy gain.

## Open questions

- Does the fingerprint ghost's put-latency gain hold across other traces and capacities? Does its hash-set cost and linear ghost-hit scan matter at larger sizes?
- Does the id-keyed index keep its get and put speedups under a baseline rerun and on other traces? Does its extra index state matter at scale?
- Is the get-path timing shift seen across many variants real, or shared environment noise? It has appeared in variants with unchanged get code in most cycles.
- Does the half-main ghost cap keep its small hit-rate gain on other workloads, and does it still help once the ghost is stored as fingerprints? Can its drain be amortized so the put cost goes away?
- Does the batched ghost window's hit-rate gain hold on other traces, and is the window close to a strict cap?
- Do the index, the counter relocation, and the ghost window compose, given they touch overlapping state? None has been tested in combination.
- Is the frequency ceiling of 3 specific to the workload where the loss was seen?
- What is the dominant put miss-path cost, if it is not the ghost position scan?
- How often does the second-chance rotation budget run out under the evaluated workloads, and does frequency aging ever change an eviction?

## Per-policy notes

- Small queue: enlarging its share alone left hit rate unchanged on the one workload reported. Earlier hit-rate gains appeared only when paired with the victim change, so the share change itself is unproven. Promotion threshold changes (`freq > 0`) showed no gain and should not be retried on current evidence.
- Main queue: the post-budget lowest-frequency victim choice showed a hit-rate effect only with the smaller small queue and did not compose with the larger one. Frequency aging left hit rate unchanged on one trace, so it is untested rather than neutral. Frequency ceiling should stay at 3.
- Ghost queue: fingerprint storage is the one ghost edit with repeated put-side benefit. Enlarging the ghost to full capacity lost hit rate. Capping it at half the main queue gained slightly, but only on one trace and only in combination with fingerprints. Batched expiry addresses drain cost, though its gain is also single-trace. Sequence tagging added cost without benefit.
- Metadata and indexing: id-keyed lookup is the lead direction across cycles. Counter relocation and index merging changed container shape without hit-rate change. Hasher choice did not help gets. Footprint should be measured at the structure level.
