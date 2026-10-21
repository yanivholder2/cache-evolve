# Variant v001 Insight

## Helped

- (none): no dimension improved. The only dimension, hit-rate, came out marginally below the parent's.

## Hurt

- hit-rate: The extra admit case (estimate within 1 of the victim's, gated on a window hit) admitted candidates that were slightly colder than the probation victims they displaced. A single in-window hit proved a weaker signal than the sketch frequency it overrode, so the swaps cost a little more than they gained.

## Takeaway

The tie-admit rule from the parent already captured most of the benefit of loosening admission. Relaxing it past ties, even with a window-hit gate, only swapped in lower-frequency entries and moved hit-rate slightly down. Per-entry in-window recency did not add useful information on top of the sketch estimate for this workload.
