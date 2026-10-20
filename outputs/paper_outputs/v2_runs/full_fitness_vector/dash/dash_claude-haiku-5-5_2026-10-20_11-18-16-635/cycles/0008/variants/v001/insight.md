# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Doubling the saturating counter's per-hit increment (+1 to +2) left hit-rate exactly equal to the parent and kept metadata at 32 bytes, so the hypothesis's expected retention gain at eviction did not show up on workload-a at size 2048. The get and put p99 differences of 1 ns and 3 ns are too small to attribute to a change that only alters a constant in an existing saturating add, so they are treated as noise. The overlay source could not be inspected in this session, so it is unconfirmed that +2 was actually applied; the flat result and the outstanding second-trace check both need to be resolved before this increment is treated as a dead end.
