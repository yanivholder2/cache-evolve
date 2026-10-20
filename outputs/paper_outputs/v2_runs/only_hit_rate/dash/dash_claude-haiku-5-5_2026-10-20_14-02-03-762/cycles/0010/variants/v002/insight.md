# Variant v002 Insight

## Helped

- None measured. The variant's fitness vector reports only hit-rate, so no dimension can be credited as improved.

## Hurt

- None measured. Hit-rate is identical to the baseline on the size-2048 workload-a trace, which is what the hypothesis predicted for a change that should not alter any promotion, landing, or eviction decision.

## Takeaway

H02 proposed folding the seen-in-stash marker into spare bits of the slot record to shrink metadata-size below the baseline's 40 bytes. The variant's fitness vector has no metadata-size or per-op-time entry, so the central claim is unverified: the size reduction was never measured. The unchanged hit-rate is consistent with a decision-preserving layout change, but it does not by itself confirm the marker was moved correctly. The overlay source could not be inspected in this session, so this insight relies on the hypothesis text and the fitness vectors alone.
