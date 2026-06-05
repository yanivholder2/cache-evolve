# Variant v003 Insight

## Helped

_(none — no dimension improved)_

## Hurt

- **hit-rate**: Routing cold items directly into the target bucket at the LRU end caused a steady stream of unproven keys to displace warm, recently-accessed items from the target tier, fragmenting the hot working set and producing more misses on workload-a's stable hot set than the baseline stash-first path.

## Takeaway

The stash is not merely an admission bottleneck — on a workload with a stable hot set it functions as a quality gate that prevents cold items from competing directly with established warm residents in the target tier. Bypassing stash on insertion shortens the theoretical promotion path from two hops to one, but it also means every cold miss immediately evicts a potentially-hot target resident, degrading locality far more than the saved hop recovers. Preserving the stash-as-admission-filter appears important for workload-a, even at the cost of the extra stash-hit requirement.
