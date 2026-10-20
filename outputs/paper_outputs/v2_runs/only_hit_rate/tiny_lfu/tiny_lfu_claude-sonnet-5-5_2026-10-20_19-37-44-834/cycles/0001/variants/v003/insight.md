# Variant v003 Insight

## Helped

- hit-rate: Essentially unchanged, with a negligible gain. Saturating 4-bit-range counters and conservative update lose almost no frequency discrimination on this trace, so shrinking the counters did not cost accuracy.

## Hurt

- metadata-size: Not measured. The fitness vector has only a hit-rate dimension, so the expected u32→u8 shrink of the sketch (4x fewer bytes) is unverified, even though `heap_bytes` was updated.
- per-op-time: Not measured, so the benefit of replacing four per-row `DefaultHasher` passes with one hash and rotate-multiply index derivation is unconfirmed. The conservative-update loop adds a second pass over the rows.

## Takeaway

Narrowing counters to u8 saturating at 15 and deriving all row indices from one hash preserved hit-rate on this workload. That suggests the sketch was over-provisioned in counter width. The variant's evaluation did not emit metadata-size or per-op-time, so the savings the hypothesis targeted cannot be credited. Those dimensions need to be measured before the change can count as a win.
