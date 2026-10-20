# Variant v001 Insight

## Helped

## Hurt

## Takeaway

H01 removes the second resident-key probe on the miss path, aiming to lower per-op-time without touching eviction order. The fitness vector reports only hit-rate, and it is flat against the parent, which fits the claim that eviction is unchanged but says nothing about the probe removal itself. Per-op-time and metadata-size are not reported, so the speedup is unmeasured. The overlay source could not be inspected in this run, so it is unconfirmed that the reuse landed in the built code.
