# Variant v001 Insight

## Helped

## Hurt

## Takeaway

H01 adds a resident-key `HashSet` probe reuse and early-return on hits, aiming to cut per-op-time while leaving eviction order unchanged. The only fitness dimension reported is hit-rate, and it is flat against the parent, which is consistent with unchanged eviction order. Per-op-time and metadata-size, the dimensions the cost claim depends on, are absent from the fitness vector, so this variant neither confirms nor refutes the speedup. The overlay source was not inspected in this run, so it is also unconfirmed that the hypothesis's edits landed; a rerun reporting per-op-time and metadata-size, plus a check of the built code, is needed before judging the change.
