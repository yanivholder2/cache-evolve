# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing the main-queue decrement-and-head-reinsert with decrement-and-tail-reinsert (capped at one pass per main-queue length) lengthens how long nonzero-frequency residents survive, but on this workload that retention displaced candidates that the baseline's head reinsert kept, and the net hit-rate went down.

## Takeaway

On workload-a at capacity 2048, giving frequency-positive main-queue victims a tail second pass did not protect warm residents better than the baseline's head reinsert; the hypothesis's premise that longer retention raises hit-rate does not hold for this trace. The metadata-size and per-op-time dimensions named in the hypothesis are absent from this variant's fitness vector, so this insight makes no claim about them.
