# Variant v001 Insight

## Helped

- hit-rate: Held exactly against the parent, confirming the hypothesis's core claim — the below-cap protected-MRU rotation is dead ordering that neither eviction (fixed front, index 0) nor demotion (only fires once saturated) ever reads, so skipping it is bit-for-bit behaviorally inert.

## Hurt

- per-op-time: The intended get-latency win never appeared — p50 stayed flat and get p99 drifted 100→300; a single-element `rotate_left(1)` is far below the 100ns timer resolution, so removing it cannot lower measured latency, and the added conditional branch on the protected-hit path coincided with a tail regression rather than a saving.

## Takeaway

Making the protected-hit rotation conditional is correct and provably inert on hit-rate, but the saved work — one element move per protected hit — is below the benchmark's 100ns measurement floor, so it cannot register as a p50/p99 improvement. This variant shows the SLRU protected-hit rotation is a valid micro-optimization target in principle, yet at bucket scale the per-hit copy is too cheap for the eval harness to reward; latency effort should move to costlier paths (the `insert(0,…)`/`remove(0)` front shifts) rather than single-element rotations.
