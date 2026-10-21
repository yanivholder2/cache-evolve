# Variant v003 Insight

## Helped

- hit-rate: Doubling the sketch width, with a flat u8 table and double-hashed row indices, spreads keys over more counters. That cuts collisions in the conservative-update estimates, so the admission comparison against the LRU victim is more accurate. Hit rate rose slightly over baseline.

## Hurt

- metadata-size: This dimension was not reported for v003, so there is no evidence on it. A 2x wider table should raise sketch memory unless the u8 packing offsets it, and the variant never demonstrated that.
- per-op-time: This dimension was not reported for v003. The hypothesis's cost goal (fewer hashes, one allocation) is unverified.

## Takeaway

The 2x wider sketch gave only a small hit-rate gain on a single trace and size. That suggests the baseline sketch was already close to collision-free for this workload, so extra width buys little. The cost half of the hypothesis cannot be judged, because the evaluation produced no size or timing data. A rerun that measures all three dimensions is needed before calling this a real improvement.
