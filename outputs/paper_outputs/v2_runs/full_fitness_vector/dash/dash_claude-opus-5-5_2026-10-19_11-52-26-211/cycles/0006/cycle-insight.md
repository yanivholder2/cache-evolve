# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle-5 v001 (two-choice stash, `s1` probed last). v001 ported stash-victim rescue. v002 added an exact per-stash-bucket borrowed counter that lets `get` skip the `s1` scan. v003 placed native target victims into holes in their probing bucket. Only v002 confirmed its hypothesis: it cut get p50 and left hit-rate exactly unchanged. v001 matched cycle-5 v002's hit-rate, but its latency claim was mostly refuted. v003's hit-rate gain was within noise.

## Patterns

- Cycle 5 blamed v002's get cost on lookup order. Cache state is the real cause. Rescue keeps normal buckets fuller, so every target/probing hit and every miss scans more occupied slots, whatever the probe order. The hit-rate vs get-latency trade from extra residency is now a cost per scan, and reordering cannot remove it.
- Lossless filters work. A small exact counter, stored per segment and not per item, skips scans with no state change and no growth in tracked metadata. Its cost lands on `put`, and mostly in the tail, because the bookkeeping re-hashes keys the insert/evict path already hashed.
- The supply of idle capacity is running out. Stash-victim rescue still pays off. Probing holes at native-demotion time almost never exist in steady state, so that source can be ruled out. Adding branches on hot paths that rarely fire costs a little latency and gains nothing.
- Every variant raised `put` cost in some form: extra rescue checks, re-hashing for the counter, or longer cascades into fuller buckets. Write-path cost is now an axis to track, not noise.

## Open questions

- Does v002's borrowed-counter gate stack with v001's rescue structure? Does it still skip most `s1` probes once rescue changes stash turnover?
- Can the counter be maintained without re-hashing, by passing `s0` or a native/borrowed flag from the caller, so that put p99 comes back?
- Can in-bucket filtering (tags/fingerprints) shorten the longer scans that rescue causes, and what would it cost in metadata?

## Continuation

New parent: v001 (rescue + `s1`-last) with v002's borrowed-counter gate, with the counter fed the already-computed `s0` / native flag so no re-hashing happens. Check that hit-rate holds and that get p50 and put p99 both improve. Then target the scan cost of fuller normal buckets with in-bucket filtering, weighing it against metadata-size. Drop native-demotion-into-probing-hole and further probe-order changes.
