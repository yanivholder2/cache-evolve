# Variant v002 Insight

## Helped

- hit-rate: `evict` now writes the SIEVE main victim's fingerprint into the ghost, so a key that main evicted slightly too early and that comes back while its fingerprint survives goes straight to the main tail. It no longer has to re-enter the 10% small FIFO and earn a second touch. The gain is real but small, and it clears the hypothesis's "strictly above v001" bar only narrowly. This means the reuse signal from main evictions is worth a little more than the small-queue fingerprints it overwrites in the shared direct-mapped table, but only a little.

## Hurt

(none) No dimension got worse. `metadata-size` is unchanged because the ghost size and struct layout are untouched. `put` latency did not move even though each main eviction now does an extra Fx hash and ghost store, which fits main evictions being a minority of misses on this trace. The `get` path was not modified, so the tiny `get` p99 shift is measurement noise and not an effect of this change.

## Takeaway

Feeding main-queue (SIEVE) victims into the same direct-mapped ghost as small-head victims is a cheap and safe change: it costs nothing in latency or memory, and it improves hit-rate only marginally. The near-zero gain suggests that few keys on this workload return soon after a main eviction, or that their benefit is mostly cancelled by ghost dilution: main victims overwrite small-queue fingerprints in the same table. This policy probe can be closed. Larger hit-rate gains are unlikely to come from deciding which eviction source feeds the ghost.
