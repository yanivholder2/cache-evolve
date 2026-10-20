# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:23:36+00:00

## timeline

### 14:23:44 thinking



### 14:23:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hit-rate gains have been rare. The only clearly positive hit-rate movement comes from a scan-resistance change that alters which entries are displaced, not how the ranking is computed (two-region bucket, probationary-first eviction).\n- Changes that keep the existing ranking intact and only alter bucket geometry have produced at most marginal gains.\n\n## Recurring failures\n\n- Aging-policy variants have not shown a measurable hit-rate gain in any cycle they appeared in. Whether the evaluation selects the aging policy, and whether its halving fires at all, is unresolved.\n- Hit-rate-only geometry and victim-scan tweaks have not paid off. Capping the victim scan left hit-rate flat, which suggests exact-minimum eviction buys little on this workload.\n- Per-op time and metadata size were not recorded in any cycle. Every cost-motivated hypothesis is therefore untested, not refuted.\n- Overlays were not verified as active. A flat hit-rate has been read as a null result without confirming the variant ran, and a Winner was declared on a flat fitness vector.\n- A frequency-gated admission check lowered hit-rate, most likely because its two frequency estimates are on different scales, so the comparison itself is suspect.\n\n## Open questions\n\n- Does the evaluation path apply and select variant overlays? This applies to the aging policy, the fingerprint filter, and the sampled victim scan.\n- Why was the fingerprint-filter variant marked a Winner when its only measured dimension did not change?\n- How often does the AgedLfu halving fire per run under the harness?\n- What are the per-op time and metadata-size costs of wider buckets, the fingerprint filter, the u8 counter, and the second region, once measured?\n- Would a shared frequency counter for incoming and resident keys make the admission gate informative?\n\n## Per-policy notes\n\n- Bucket geometry (fixed-width buckets): a marginal hit-rate gain with an unmeasured per-op scan cost. Bucket-conflict misses are a minor share of total misses, so further geometry changes have limited headroom.\n- Bucket policy refactor (static dispatch): hit-rate neutral as expected. Get-latency effect is unmeasured.\n- AgedLfu (aging and halving): no measurable hit-rate gain across the aging-related variants (a u8 saturating age counter and a cache-wide insert-count halving trigger). Activation under the harness is unconfirmed. Re-test only after activation is verified.\n- Fingerprint filter (per-bucket): declared a Winner with flat hit-rate. Per-op savings are untested.\n- Victim selection (sampled scan capped at 8 slots): flat hit-rate. Scan-cost savings are a candidate only if measured.\n- Two-region bucket (probationary-first eviction): the only hit-rate improvement so far. Metadata and per-op costs are unknown.\n- Frequency-gated admission: lowered hit-rate. Drop unless the two frequency estimates are placed on a common scale first.\n"
}
```

### 14:23:51 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0003. It has the four required sections in order, built only from the inlined ledger digest. I did not open the previous insights file.

The main finding is that the ledger shows few real wins:

- **Recurring wins:** Only one change has moved hit-rate clearly upward: the two-region bucket that evicts from the probationary region first. Bucket-geometry changes have produced at most marginal gains.
- **Recurring failures:** The aging-policy variants have shown no measurable hit-rate gain in any cycle. Per-op time and metadata size were never recorded, so every cost-motivated hypothesis is untested rather than refuted. Flat hit-rate has also been read as a null result without confirming the overlay was active, and cycle 0002 declared a Winner on a flat fitness vector.
- **Open questions:** Whether the evaluation applies and selects variant overlays, how often the AgedLfu halving fires per run, and the per-op and metadata costs of the variants once they are measured.
- **Per-policy notes:** One entry per policy family, grouped as the template asks.

I did not invent numbers and described all dimensions qualitatively. The "Recurring wins" section is thin because the digest contains no pattern that has held across several cycles.


---

- finished: 2026-10-20T14:23:52+00:00
- duration: 15.49s
- status: ok
- input_tokens: 4
- output_tokens: 2967
- cache_read_tokens: 35723
- cache_write_tokens: 8803
- cost_usd: 0.003602
