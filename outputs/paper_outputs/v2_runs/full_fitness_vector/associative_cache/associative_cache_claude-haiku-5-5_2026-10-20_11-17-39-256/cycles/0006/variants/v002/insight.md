# Variant v002 Insight

## Helped

- per-op-time: Skipping the access-counter store on get hits once the counter is saturated removes a write from most hot-set hits, and the get tail latency dropped, which fits the hypothesis that this store was the get-side cost.

## Hurt

- per-op-time: The put median shifted by a hair, but the put path and victim scan are untouched by this change, so the shift is run-to-run noise rather than a cost of the store-skip.

## Takeaway

Hit-rate and metadata stayed identical to the parent, as the cap-gated increment predicted, so eviction still sees the same counter values. The gain is concentrated in the get tail rather than the median, which suggests the saturated-counter store was an occasional stall on the hit path more than a steady per-hit cost. Because put-side tail numbers moved without any put-path change, single-run tails are noisy here, so the get tail gain should be confirmed with repeated runs before it is treated as real.
