## Recurring wins

- Hit-rate has moved positively only when a change alters which entries get displaced (scan-resistant probationary-first eviction in Cycle 0003, shared-frequency admission gate in Cycle 0004), not when it tweaks how the existing ranking is computed.
- Capping or sampling the exact-minimum victim scan left hit-rate flat in two cycles (Cycles 0003 and 0004). This is a plausible cost-side saving, but it is unmeasured.
- Putting incoming and resident keys on one frequency scale appears to fix the admission-gate loss seen in Cycle 0003. One cycle of evidence so far.

## Recurring failures

- Cost-motivated changes have been judged on hit-rate alone in every cycle (1–4). Per-op time and metadata size were never recorded, so no cost claim has been tested. This is the most persistent gap.
- Aging-based LFU variants (periodic halving, saturating counters, insert-count triggers) produced no measurable hit-rate gain in Cycles 1 and 2. Whether the evaluation even selects or fires the aging policy is unverified.
- Stacking a second reuse tier on probationary-first eviction lowered hit-rate (Cycle 4). The failure looks tied to the protection cap and demote-on-promotion path rather than the idea of a second tier.
- Admission gating without a common frequency scale lowered hit-rate (Cycle 3). Treat that as a flawed comparison, not a verdict on admission.
- Hit-rate-flat results cannot be read as nulls until overlay activation is confirmed. Cycles 3 and 4 both had variants whose overlays could not be shown to run.

## Open questions

- Which dimension made the Winner in Cycles 2 and 4? In both, the labeled Winner had no hit-rate gain, while another variant did (Cycle 4). Is the Winner rule using a metric missing from the insights?
- Is the shared-frequency admission gain reproducible beyond workload-a, and what metadata or per-op cost does its shared counter add?
- Do the cost-side savings claimed for the fingerprint filter, the saturating u8 counter, the static-dispatch refactor, and the wider buckets show up once per-op time and metadata size are measured?
- Does the AgedLfu halving fire at all under the harness, and how many halvings occur per run?
- Does a protected tier work if the cap or demotion rule changes, rather than the promotion path alone?

## Per-policy notes

- Probationary-first / two-region eviction: positive in Cycle 3; a stacked second-hit protected tier was negative in Cycle 4. The base idea is the strongest hit-rate lead so far.
- Bounded or sampled victim scan: hit-rate flat in Cycles 3 and 4. Only a cost measurement can justify it.
- Frequency-gated admission: negative when the two estimates were on different scales (Cycle 3); positive with a shared counter (Cycle 4).
- AgedLfu and its variants (periodic halving, u8 saturating counters, insert-count trigger): no measurable hit-rate gain across Cycles 1 and 2. Activation is unverified.
- Bucket geometry: fixed-width buckets (Cycle 1) gave a marginal hit-rate gain with unmeasured scan cost. Bucket-conflict misses look like a small share of total misses at this cache size, which caps geometry gains.
- Static dispatch on the bucket policy (Cycle 1): hit-rate neutral by design; latency effect unmeasured.
- Per-bucket fingerprint filter (Cycle 2): hit-rate flat; the Winner label is unsupported by the cited evidence. Keep only with a per-op-time measurement.
