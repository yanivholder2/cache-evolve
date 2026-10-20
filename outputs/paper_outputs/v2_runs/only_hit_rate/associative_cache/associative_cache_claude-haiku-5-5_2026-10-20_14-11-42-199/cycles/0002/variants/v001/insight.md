# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The hypothesis adds a per-bucket 8-bit fingerprint filter in front of the full-key comparison, a change that should move only per-op time and leave hit-rate exactly where the parent put it, since eviction and victim selection are untouched. The hit-rate dimension did not move, which is what the change predicts, but this fitness vector records only hit-rate, so the latency claim that motivated the change is still untested. The vector is also identical to the parent's, so it is unclear whether the evaluation ran against the fingerprint code at all; a per-op-time measurement on the same trace is needed before this variant's effect can be judged.
