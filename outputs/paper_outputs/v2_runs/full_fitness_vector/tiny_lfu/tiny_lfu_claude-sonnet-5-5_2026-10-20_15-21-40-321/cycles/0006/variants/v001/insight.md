# Variant v001 Insight

## Helped

- metadata-size: The `TinyLfuCache` struct itself got slightly smaller, and `Node` and the sketch heap were unchanged as the hypothesis required. The sketch is not a likely cause. I could not read the overlay source in this session, so the reason for the smaller struct is unverified.

## Hurt

- per-op-time: The hypothesis predicted a speedup from computing one hash per call and slicing it into row indices. `get` p50 and p99 both got worse against the parent, and `put` stayed about flat. The single-hash derivation did not stack on this lineage. Any hashing saved was smaller than the cost of the new index derivation or than measurement noise.
- hit-rate: It dropped marginally. Row indices sliced from one hash are correlated, so a collision in one row is more likely to repeat in the others. That weakens the count-min minimum and adds a little noise to the strict `>` admission test. The gap is tiny and could also be noise.

## Takeaway

Replacing per-row hashing with a single hash and shift-derived indices did not make this incumbent faster. The parent's sketch hashing was evidently not the bottleneck for `get`, and the speedup seen on the baseline lineage did not carry over. Hit-rate was held roughly constant, as the hypothesis wanted. Further time work should profile where `get` spends its time before touching the sketch again.
