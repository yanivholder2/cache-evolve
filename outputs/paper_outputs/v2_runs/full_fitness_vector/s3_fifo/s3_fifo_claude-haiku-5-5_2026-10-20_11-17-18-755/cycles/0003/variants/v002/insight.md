# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Frequency aging on rotation (decrementing the counter of main-queue entries passed over by the second-chance loop) left hit-rate on workload-a at 2048 unchanged from the parent, so this change did not alter any eviction outcome that the trace could detect, which is the outcome the hypothesis predicted for hit-rate. The timing movements in the fitness vector are not attributable to this diff: `get()` was not modified, and the decrement adds only one store per rotated entry on the put path, which would not plausibly speed up put. This insight was written without access to the overlay source (no directory listing or source read was possible in this run), so the description of the change relies on the hypothesis file alone.
