# Variant v002 Insight

## Helped

- hit-rate: Halving the ghost queue capacity (to `capacity / 2`, minimum 1) gave a small gain over parent v001. Fewer remembered evictions means fewer keys re-admitted straight into main on a ghost hit, so admission is stricter, as the hypothesis predicted. The overlay source was not available for inspection, so this attributes the gain to the hypothesis's described change.

## Hurt

- None observed. Only hit-rate was measured, so metadata-size and per-op-time effects are unknown. A shorter ghost memory could hurt workloads with long reuse distances, but this evaluation did not test that.

## Takeaway

Ghost length is a second tunable admission lever, and shortening it moves hit-rate in the opposite direction from the cycle-2 lengthening that hurt. The gain is small and comes from one trace at one cache size, so it may be within noise of the trace and not a general result. Combined with the smaller small queue, it suggests this workload rewards stricter admission.
