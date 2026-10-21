# Variant v003 Insight

## Helped

- hit-rate: Swapping strict LRU for CLOCK with unreferenced insertion and a conservative-update sketch admission gate beat the baseline on workload-a. Rejecting one-hit-wonders whose sketch estimate is below the victim's keeps scan traffic from displacing reused entries. This is the frequency-aware scan resistance that pure recency lacks.

## Hurt

- metadata-size: Not measured, so the effect is unknown. The variant's fitness vector has no entry for this dimension. The change adds a 4-row sketch and a per-entry reference bit, and it drops the list-link overhead. The net effect is untested.
- per-op-time: Not measured. The hypothesis expected cheaper hits, since a hit now only sets a bit. The vector contains no timing data to confirm it. Misses now pay for four sketch updates plus a hand sweep.

## Takeaway

CLOCK with a sketch admission gate improved hit-rate over strict LRU on this trace. The hypothesis's other claim, cheaper hits, cannot be judged because the evaluation recorded only hit-rate. I could not inspect the overlay source in this session, so this insight rests on the hypothesis text and the fitness vectors. The hit-rate gain therefore cannot be traced to specific code. A follow-up should measure metadata size and per-op time before treating this as a clear win.
