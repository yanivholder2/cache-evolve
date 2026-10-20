# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Doubling the ghost-set retention bound lets keys stay admissible for a longer re-reference gap, so more keys that never recur are admitted into the arena and displace entries that would have hit, a net loss on workload-a at size 2048.

## Takeaway

The loss, though small, shows the ghost filter is active on this trace: changing only its retention bound changed the outcome, which contradicts the "never confirmed engaged" note. Longer ghost memory is not the missing ingredient, so the capacity-sized bound in the parent is at least as good here and a longer memory is the wrong direction for this workload.
