# Variant v002 Insight

## Helped

- hit-rate: Capping the ghost queue at `main_size / 2` (768 slots vs. the baseline's 1536) tightened the re-access window so that ghost hits correspond only to keys evicted very recently, eliminating stale entries that were matching noise rather than genuine recurring accesses in workload-a.

## Hurt

_(none)_

## Takeaway

For workload-a's LIRS trace, the effective re-use distance fits comfortably within a 768-key ghost window, so the outer half of the baseline ghost queue was holding eviction history that produced false promotion signals rather than useful ones. Halving the ghost cap — while leaving small-queue size and the promotion threshold (`freq > 1`) unchanged — was sufficient to improve hit rate, confirming that ghost precision matters more than ghost coverage for this workload.
