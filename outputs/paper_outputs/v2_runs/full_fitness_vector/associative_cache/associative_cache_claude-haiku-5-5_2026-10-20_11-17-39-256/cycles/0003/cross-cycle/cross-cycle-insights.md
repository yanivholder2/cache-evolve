## Recurring wins

- Re-reference signals beat recency-only ordering. Keeping keys that have been hit again protects them from bursts of one-shot inserts, which is the pollution pattern the bucket-local work keeps targeting. Seen with frequency-based victim selection (cycle 2) and second-chance reference bits (cycle 3), which use different state for the same effect.
- The second-chance variant is the first result to improve hit-rate and put cost together, so it is the current base to build on.

## Recurring failures

- Moving items within a bucket (midpoint promotion, midpoint reinsertion) lost hit-rate on workload-a. Do not revisit as specified.
- Bucket-local ghost memory has not paid off. A short ring (cycle 1) and a longer ring (cycle 2) both failed to lift hit-rate, so the reinsertion idea looks weak, not just under-sized. Do not tune ring length further.
- Extra lookup layers add cost without hit-rate gain. The fingerprint prefilter on 32-slot buckets added a second scan on get and write work on every put, with no hit-rate change.
- Put-side victim-search shortcuts have not worked. A cached minimum is invalidated by get-hit counter updates. A merged scan with per-eviction halving does not help. Extra miss-driven decay erased frequency protection too fast. Periodic halving is the working aging schedule and should stay.

## Open questions

- Does re-reference retention hold on traces other than workload-a? Every winning signal so far has only been measured there.
- Is the second-chance gain driven by the reference bit itself? The overlay was unreadable in cycle 3, so the attribution rests on the hypothesis and metadata delta, not on code.
- Can per-item state shrink to one bit without losing the hit-rate gain? Can the get-side write on hits be avoided, for example by setting reference state only on some hits?
- Can the frequency or reference victim search get cheaper, for example by considering fewer ways, without losing hit-rate?
- Does the small wider-bucket hit-rate gain (cycle 1, single cycle) hold on other traces? It is the only layout result that improved hit-rate and is worth re-measuring before building on.
- Put and get latency shifts have been single-run and partly unexplained by the diffs. Treat any timing gain as uncertain until re-measured.
- The metadata-size probe has not reported any change in any variant, including ones that clearly add per-bucket or per-slot state. Storage cost is unmeasured until the probe counts that state.

## Per-policy notes

- Bucket width / geometry layout: the only hit-rate gain seen was small, and the longer linear scan on get was the cost. Re-measure before building on it.
- Midpoint promotion and reinsertion: negative on hit-rate and add shifting work on the hot path. Drop as currently specified.
- Bucket-local ghost memory: no gain at short or longer ring length. Deprioritize.
- Fingerprint prefilter: no hit-rate change, extra get scan and put writes. Not worth carrying forward.
- Frequency-based victim selection (per-slot access counters, periodic halving): retains re-referenced keys through one-shot bursts. Its put-side full-bucket scan is the open cost. Keep the periodic halving schedule. Cached-minimum and merged-halving variants failed.
- Second-chance reference bit: hit-rate up and put cost down, with slightly higher get cost from a per-hit write and metadata growth beyond one bit. Next steps: confirm the mechanism, reduce per-item state toward one bit, and cut the get-side write.
