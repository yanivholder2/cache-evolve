## Recurring wins

- Retention of re-referenced keys is the only direction that has improved hit-rate. Wider buckets (cycle 1) and frequency-based victim selection (cycle 2) both helped by keeping reused items against one-shot inserts. The gains are small so far.
- The frequency signal beat recency-only ordering where recency had not helped. This is the strongest hit-rate result in the ledger.

## Recurring failures

- Moving items within a bucket on hit or reinsertion (midpoint promotion, midpoint reinsertion) lowered hit-rate on workload-a and added shifting work to the hot path.
- Bucket-local ghost memory has not lifted hit-rate in either form tried: a small ring (cycle 1) and a longer ring (cycle 2). The reinsertion idea looks weak here, not just undersized.
- Extra linear scans on the get path cost time without hit-rate gain. This came from a fingerprint prefilter (cycle 2) and from wider buckets' longer scan (cycle 1).
- Per-op latency regressions have followed each structural change. Put-side full-bucket victim scans are the current source.
- Metadata-size has not moved in any variant, including ones that add per-bucket or per-slot state. The probe is not measuring new state and cannot be used to judge storage cost.

## Open questions

- Do the hit-rate gains from wider buckets and frequency victim selection hold on traces other than workload-a, or are they tied to its one-shot insert pattern?
- Can frequency victim selection be made cheap (running minimum, fewer ways) without losing its hit-rate gain?
- What should the size probe count so per-bucket and per-slot state shows up?
- Put and get latency shifts have been single-run and sometimes unexplained by the diffs. Are they noise or real layout effects?

## Per-policy notes

- Frequency-based victim selection: the only hit-rate winner so far. Its put-side full-bucket scan is the main cost. Build on it and cut that cost first.
- Wider buckets (geometry layout): small hit-rate gain with a longer get-path scan. Re-measure on more traces before building on it.
- Midpoint promotion and midpoint reinsertion (recency-position policies): hurt hit-rate and added hot-path shifting. Dropped as specified.
- Bucket-local ghost memory: no hit-rate lift at either ring length. Do not tune ring length further.
- Fingerprint prefilter (lookup filter, not an eviction policy): no hit-rate change, added get and put work. Dropped.
