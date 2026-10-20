# Variant v001 Insight

## Helped

- hit-rate: Held at the parent's value, as required. The hasher swap only changes bucket placement, not eviction, promotion, ghost bounds or queue ratios.

## Hurt

- per-op-time: Both `get` and `put` got slightly slower at the median, and the `put` tail also regressed. The cheaper multiply-xorshift hasher did not beat the parent's hashing cost on this workload. A plausible cause is poorer bucket distribution, which lengthens probe chains and offsets the saved hashing cycles. I could not read the overlay source, so this is inferred from the hypothesis and the fitness vectors.
- metadata-size: Unchanged, so the hasher swap did not affect it. The parent's metadata overhead stays.

## Takeaway

Replacing the hasher on the already-split index and ghost map did not lower per-op latency. It moved every latency figure in the wrong direction by a small amount. Hashing was not the bottleneck on the parent's `get` and `put` paths. A weak finalizer can cost more in collisions than it saves in hash computation. The parent's split-ghost design remains the better base.
