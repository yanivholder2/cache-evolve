# Variant v003 Insight

## Helped

- per-op-time: shrinking the ghost bound to `(main_size / 4).max(1)` makes every `put` cheaper, because the linear `ghost.iter().any(...)` membership scan and the `retain` on ghost-hit now traverse a queue a quarter as long — `put` p50/p99 latency roughly halved.

## Hurt

- hit-rate: the hypothesis predicted tightening the re-admission window would raise hit-rate, but it fell — quartering the ghost discards keys whose reuse, while not immediate, was still near enough to benefit from promotion, so genuine second-chance admissions are lost rather than sharpened.

## Takeaway

On workload-a the full-`main_size` ghost was not diluting the main queue with stale far-reuse keys; its longer tail was actually catching legitimate near reuse, so cutting it to ~25% trades a small hit-rate loss for a large `put`-latency win. The ghost length is a latency/hit-rate lever here, not a free hit-rate gain — the reuse distance this trace rewards extends past the freshest quarter of evictions.
