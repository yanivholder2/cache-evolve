# Variant v003 Insight

## Helped

- hit-rate: Retaining re-referenced keys through one-shot bursts, the signal H03 targets with a second-chance reference bit, moved the workload-a hit-rate clearly above the baseline.
- per-op-time: Put cost dropped against baseline, which fits a bounded second-chance scan replacing the baseline's full recency-ordered victim choice on a full bucket.

## Hurt

- metadata-size: Each stored item grew in size against baseline, which is wider than the one-bit-per-slot state H03 specifies and points to a multi-bit per-item field.
- per-op-time: Get cost rose slightly against baseline, consistent with the extra write to a reference state on every get hit.

## Takeaway

The gain on hit-rate and the put-side saving are the expected effects of re-reference-aware victim selection, but the metadata growth suggests the implementation kept a multi-bit per-item field rather than the single reference bit the hypothesis describes. This insight is limited: the source overlay could not be read in this session, so the code-level attribution above is inferred from the hypothesis, the fitness vectors, and the metadata delta rather than confirmed from the diff. Verify the overlay before relying on the mechanism claim.
