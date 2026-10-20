# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Cutting ghost capacity from `main_size / 2` to `main_size / 4` (~461 slots for a 2048-slot cache) discarded ghost entries in positions 461–922 that still carried genuine re-use signal for workload-a, so keys that would have been ghost-promoted directly into main were instead admitted cold through small and had to re-earn main residency.

## Takeaway

Workload-a's re-use distance is long enough that a non-trivial fraction of re-accesses arrive more than 461 evictions after displacement — squarely in the range that `main_size / 4` discards. This means v003's `main_size / 2` ghost cap was not over-broad; it was capturing real signal at the outer half of the window. Further halving the ghost size crossed below the workload's natural re-access horizon, making the ghost noisier in the wrong direction by missing hits rather than filtering cold entries. The result establishes `main_size / 2` as approximately the right precision point for this workload, and suggests ghost size reductions below that level are counterproductive.
