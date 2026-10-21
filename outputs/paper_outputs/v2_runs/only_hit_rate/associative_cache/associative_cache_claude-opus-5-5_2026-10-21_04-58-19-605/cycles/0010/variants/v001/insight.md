# Variant v001 Insight

## Helped

(No measured dimension improved.)

## Hurt

- hit-rate (flat, a negligible dip well inside H01's ±0.001 "unchanged" band, not a real regression): putting a splitmix64 finaliser and high-bit multiply-shift reduction on the bucket index did not cut misses. Re-seeding `FpHasher` with the `0x9e37_79b9_7f4a_7c15` salt only changes which 16-bit ghost fingerprints collide, and that can only nudge the 0-vs-1 admission counter. Together the two changes reshuffled which keys share a bucket without changing how many conflict misses the 128-bucket GCLOCK/LIP layout takes on workload-a at 2048.

## Takeaway

H01's refutation case holds. The parent's bucket index was already balanced well enough that no hot-key pile-up is left for a better-mixing single hash to fix. Workload-a's remaining misses at 2048 entries are capacity or in-bucket replacement misses, not hash-skew conflicts. Since alternate placement is forbidden and hash quality is now ruled out, the "which bucket" lever is used up for this trace. The finaliser adds a few multiplies per lookup for no hit-rate gain, so v001 does not beat its parent as a base. Caveat: the container's bucket-index source could not be located in the overlay with the read-only tools available. The finaliser change is therefore taken from the hypothesis, while the ghost-hasher salt was checked in `associative_cache_bucket.rs`.
