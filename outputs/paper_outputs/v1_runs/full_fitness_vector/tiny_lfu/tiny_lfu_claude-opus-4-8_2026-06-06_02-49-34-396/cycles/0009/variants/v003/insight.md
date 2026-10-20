# Variant v003 Insight

## Helped

- hit-rate: Confirmed neutral exactly as predicted — `saturating_add(1)` caps at `MAX_COUNT == u8::MAX`, so stored counts, every `estimate`, and every admission/eviction decision are bit-identical to the parent; the hit-rate is unchanged, validating the equivalence half of the claim.

## Hurt

- per-op-time: The branchless saturating bump made the hot path *slower*, not faster — get and put p50/p99 both rose versus the parent, disproving the core claim. On this saturating, high-frequency counter distribution the data-dependent `if *counter < MAX_COUNT` branch was already near-perfectly predicted, so there was no misprediction to eliminate; `saturating_add` instead forces an unconditional read-modify-write (and overflow-check codegen) where the branch let the common path skip the store.

## Takeaway

Swapping the conditional increment for `u8::saturating_add` is value-preserving but a per-op-time regression: the counter guard this sketch removes is one the branch predictor handles essentially for free, so the "branchless = cheaper" intuition is inverted here. The increment loop body is not a profitable cost surface — future per-op gains should target loop *count* (sketch depth) or the eviction walk, not the per-row bump.
