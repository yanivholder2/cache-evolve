# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Moving admission from front (`insert(0, ..)`) to midpoint (`insert(len/2, ..)`) let unpromoted scan items sit *above* index 0, so they survive several evict-and-insert rounds before sinking out — the longer probation window kept dead scan items in the bucket longer and displaced genuinely-hot items clustered near the back, dropping hit-rate below the parent's pure-front admission.
- per-op-time: Inserting at the bucket midpoint shifts roughly half the `Vec` per miss instead of the whole tail from index 0, and the extra churn from scan items lingering longer nudged the `get`/`put` p99 tails up even though the p50 summary held.

## Takeaway

Midpoint probation is strictly worse than front admission here: the extra proving window it grants hot-but-not-yet-rehit items is dwarfed by the cost of letting never-reused scan items linger above the eviction slot. The front slot's value is precisely that a fresh, unproven miss is the *next* thing evicted — pushing it to the middle trades away that scan-resistance, so on the admission-geometry axis, front is the better stateless point than midpoint.
