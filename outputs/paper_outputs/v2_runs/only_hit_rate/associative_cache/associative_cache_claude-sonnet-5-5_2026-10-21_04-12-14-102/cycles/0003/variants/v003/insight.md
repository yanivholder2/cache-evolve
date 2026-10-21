# Variant v003 Insight

## Helped

- hit-rate: Replacing the baseline's plain move-to-back LRU with a per-bucket 2Q-style probation/protected split makes one-touch entries from scans and loops evict each other in probation. Entries hit twice are promoted and so survive in the protected region, and this gave a clear gain on the LIRS trace. The ghost-ring adaptation of the probation target `p` may have contributed, but this evaluation doesn't isolate it.

## Hurt

- metadata-size: Not measured in this variant's fitness vector, so there is no evidence either way. The change adds per-bucket state (`p`, an 8-key ghost ring) and a per-entry probation/protected marker, so metadata should have grown.
- per-op-time: Also not measured. Promotion on a second hit, ghost-ring lookups on misses and the two-region eviction scan add work to `get` and `put` on top of the baseline's single move-to-back.

## Takeaway

Adding a segmented (2Q-like) policy inside each hashed bucket, with no change to placement or capacity, is enough to beat plain LRU on the LIRS workload. The fitness vector only reports hit-rate, so the cost of the added bookkeeping in metadata and latency is unknown. A later variant should be evaluated on all dimensions. It should also test a fixed `p` against the adaptive one to see whether the ghost-ring tuning earns its extra state.
