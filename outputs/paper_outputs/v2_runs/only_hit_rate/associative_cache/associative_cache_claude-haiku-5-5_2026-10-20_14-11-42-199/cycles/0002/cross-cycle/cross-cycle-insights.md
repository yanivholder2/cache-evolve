## Recurring wins

- No approach has yet shown a repeatable gain. The two Winner labels (Cycle 0001 v001, Cycle 0002 v001) rest on hit-rate movement that was marginal or flat.

## Recurring failures

- Hit-rate-only changes have not moved the metric. Aging-policy variants returned no measurable gain in two separate cycles.
- Cost-side claims (per-op time, metadata size, get latency) have gone unmeasured in every variant across both cycles, so none of those hypotheses has been tested.
- Winner labels are not supported by the recorded evidence when the only measured dimension is flat.

## Open questions

- Does the evaluation path actually select the aging policy and the fingerprint code, or are those changes absent from the build it runs?
- How often does the AgedLfu halving fire per run?
- Do per-op-time savings (fingerprint filter) and metadata-size savings (u8 counter) appear once those dimensions are measured?
- Does the static-dispatch refactor reduce get latency when measured separately from put?

## Per-policy notes

- Bucket geometry (fixed-width buckets): the one Winner so far, with a small hit-rate gain. Its scan-cost side is unmeasured. Bucket-conflict misses look like a minor share of total misses, which caps what geometry changes can gain.
- Bucket policy, fingerprint filter: flat hit-rate; per-op-time benefit is a hypothesis only.
- Bucket policy, static-dispatch refactor: expected neutral on hit-rate; latency effect unmeasured.
- AgedLfu (aging period, counter width, insert-count trigger): two null hit-rate results across cycles. Activation and halving frequency are unverified, so the policy itself has not been fairly tested.
