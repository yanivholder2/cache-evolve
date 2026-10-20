# Variant v003 Insight

## Helped

- per-op-time: Tail latency on put and get came out slightly lower, and the median is essentially unchanged. The change adds no work to the hot path (the ghost bound is a constant), so the tail difference is most likely measurement noise and not a real gain.

## Hurt

- hit-rate: Doubling the ghost bound to 2x main made the ghost queue remember more evicted keys. On workload-a that did not catch useful extra re-references, and hit rate fell slightly. The likely cause is that stale ghost hits let one-hit-wonders into main, which crowds out objects that deserve to stay.
- metadata-size: The S3Fifo struct grew, most likely from the extra field that stores the precomputed ghost bound. The per-entry size is unchanged, so the headline metadata figure did not move.

## Takeaway

Lengthening the ghost history beyond the main-size window did not pay off on this workload. Reuse distances past main capacity appear to be rare, and the extra ghost hits promote marginal objects. The baseline ghost sizing looks near a sweet spot, so more ghost history is not a free hit-rate gain.
