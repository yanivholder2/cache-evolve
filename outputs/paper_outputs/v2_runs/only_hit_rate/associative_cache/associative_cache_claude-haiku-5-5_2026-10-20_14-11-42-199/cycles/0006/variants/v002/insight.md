# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Placing each newly admitted entry at index 0 of the probationary slice, instead of at its end, dropped workload-a hit-rate below the parent's, so the eviction-next placement cost hits rather than saving them.

## Takeaway

Inserting new misses at the front of probation is the opposite of what helped in v001: admission gating filtered one-shot keys, but this change made every admitted key the first probation entry, so it lost the window it needed to be re-referenced before eviction. The hypothesis assumed index 0 is the eviction candidate, but the bucket diff only shows the placement, and the eviction policy that picks the probation victim is not part of this change, so that assumption is unverified here.
