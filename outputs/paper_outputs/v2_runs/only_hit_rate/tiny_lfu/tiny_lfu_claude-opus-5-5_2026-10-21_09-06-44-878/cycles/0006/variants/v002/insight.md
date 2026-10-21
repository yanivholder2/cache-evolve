# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Halving `SKETCH_WIDTH_FACTOR` (16 → 8) roughly doubles collisions in each row. Conservative update in `increment_hashed` only bumps the counters equal to the key's minimum, which cancels most but not all of the extra overestimation. As a result, a few `estimate_hashed` values came out different from the parent's. In this SLRU design those values feed two strict comparisons: the `<` scan over the `VICTIM_SAMPLE` oldest probation entries and the `>` admission contest. A one-count shift is enough to flip a victim choice or an admission. The loss is tiny, but it is real (the simulation is deterministic). Unlike the cycle-0005 non-SLRU case described in the hypothesis rationale, the narrow sketch did not reproduce its parent's decisions exactly. The `u16` narrowing is not the cause: low-frequency counts on this trace come nowhere near `u16::MAX`, so the saturation check never fires.

## Takeaway

Combining the 8× `u16` conservative-update sketch with the SLRU winner is nearly free in hit-rate. However, it is not decision-for-decision identical to the parent the way it was outside SLRU. Probation-only victim sampling compares a newcomer against a few low-count probation entries, so any residual collision error matters more than it did before. The 4× sketch-memory cut the hypothesis aimed for is not in this fitness vector: only hit-rate was scored, so whether the trade pays off depends on a metadata-size dimension that wasn't measured here. Width 8× looks close to the floor where collision error starts to show up in this design. Shrinking further would probably need an extra tie-breaker, such as recency between equal estimates, rather than relying on conservative update alone.
