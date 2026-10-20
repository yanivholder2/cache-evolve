## Recurring wins

- None yet confirmed across cycles; the ledger holds one cycle so far.
- Widening buckets is the only change with a hit-rate gain on workload-a, but the gain was small and the get path paid for a longer linear scan. Treat it as a single-cycle signal until re-measured.

## Recurring failures

- Moving items within a bucket (midpoint promotion of front-half hits, midpoint reinsertion) lost hit-rate on workload-a and added shifting work on the hot path. Seen in more than one variant, so the approach looks weak as specified.
- Bucket-local memory (back-half protection, small ghost ring) may have windows too short for workload-a's reuse distances, so it did not pay off. Not yet proven: the window-length explanation is a suspicion, not a measured result.

## Open questions

- Does the wider-bucket hit-rate gain hold on other traces, or is it specific to workload-a's conflict pattern?
- Is the ghost ring too small to matter, or is ghost-based reinsertion weak on this trace? A longer ghost window or a hit-count gate would separate these.
- What drives the unexplained put-side and get-side latency shifts: noise or layout effects? Put-side numbers need re-verification before any variant is judged on them.
- Does the metadata-size probe cover the new per-bucket state? It reported no change in any variant, which is suspicious.

## Per-policy notes

- Bucket geometry (wider buckets): hit-rate up slightly; get-path cost up from longer scans. Keep the direction and re-measure before building on it.
- Positional policies (midpoint promotion, midpoint reinsertion): hit-rate down on workload-a and extra hot-path work. Drop as currently specified.
- Ghost-memory policies (bucket-local ghost ring, back-half protection): no hit-rate gain yet. Test with a longer ghost window or a hit-count gate before dropping.
