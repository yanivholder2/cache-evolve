## Recurring wins

- Hit-rate gains have been rare. The only clearly positive hit-rate movement comes from a scan-resistance change that alters which entries are displaced, not how the ranking is computed (two-region bucket, probationary-first eviction).
- Changes that keep the existing ranking intact and only alter bucket geometry have produced at most marginal gains.

## Recurring failures

- Aging-policy variants have not shown a measurable hit-rate gain in any cycle they appeared in. Whether the evaluation selects the aging policy, and whether its halving fires at all, is unresolved.
- Hit-rate-only geometry and victim-scan tweaks have not paid off. Capping the victim scan left hit-rate flat, which suggests exact-minimum eviction buys little on this workload.
- Per-op time and metadata size were not recorded in any cycle. Every cost-motivated hypothesis is therefore untested, not refuted.
- Overlays were not verified as active. A flat hit-rate has been read as a null result without confirming the variant ran, and a Winner was declared on a flat fitness vector.
- A frequency-gated admission check lowered hit-rate, most likely because its two frequency estimates are on different scales, so the comparison itself is suspect.

## Open questions

- Does the evaluation path apply and select variant overlays? This applies to the aging policy, the fingerprint filter, and the sampled victim scan.
- Why was the fingerprint-filter variant marked a Winner when its only measured dimension did not change?
- How often does the AgedLfu halving fire per run under the harness?
- What are the per-op time and metadata-size costs of wider buckets, the fingerprint filter, the u8 counter, and the second region, once measured?
- Would a shared frequency counter for incoming and resident keys make the admission gate informative?

## Per-policy notes

- Bucket geometry (fixed-width buckets): a marginal hit-rate gain with an unmeasured per-op scan cost. Bucket-conflict misses are a minor share of total misses, so further geometry changes have limited headroom.
- Bucket policy refactor (static dispatch): hit-rate neutral as expected. Get-latency effect is unmeasured.
- AgedLfu (aging and halving): no measurable hit-rate gain across the aging-related variants (a u8 saturating age counter and a cache-wide insert-count halving trigger). Activation under the harness is unconfirmed. Re-test only after activation is verified.
- Fingerprint filter (per-bucket): declared a Winner with flat hit-rate. Per-op savings are untested.
- Victim selection (sampled scan capped at 8 slots): flat hit-rate. Scan-cost savings are a candidate only if measured.
- Two-region bucket (probationary-first eviction): the only hit-rate improvement so far. Metadata and per-op costs are unknown.
- Frequency-gated admission: lowered hit-rate. Drop unless the two frequency estimates are placed on a common scale first.
