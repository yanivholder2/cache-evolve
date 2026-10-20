# Variant v001 Insight

## Helped

## Hurt

- per-op-time: The get path gained a 32-byte fingerprint pass that runs before the full-key compare, so each lookup now does two linear scans of the bucket instead of one. At 32 slots per bucket, the prefilter does not save enough key compares to pay for its own scan.
- per-op-time: Insertion, eviction, and the remove-and-push promotion path now also have to write the bucket's fingerprint array, adding work to every put and to every get hit that promotes. The put side regressed more than the get side, which is consistent with this extra write work.

## Takeaway

A one-byte fingerprint prefilter does not recover get latency in a 32-slot bucket. The array is itself a linear scan, and every write path now maintains it, so the change adds cost on both get and put. Hit-rate held exactly, as expected since eviction and promotion order were untouched, which isolates the regression to the fingerprint layer itself. The metadata-size probe reports the same bytes as the parent, so it does not capture the per-bucket fingerprint storage and should not be read as evidence that the extra state is free.
