# Variant v003 Insight

## Helped

(No measured dimension improved. The eval covers hit-rate only. Halving `ghost_ring` to `(main_size / 2).max(1)` and shrinking `ghost_map`'s initial capacity to match does halve ghost metadata, but metadata-size is not in the fitness vector, so that saving is unmeasured.)

## Hurt

- hit-rate: Main victims and small rejects still share one FIFO ring. Halving it drops the older half of ghost history, and that half includes keys whose reuse distance falls between about `main_size / 2` and `main_size` fingerprint insertions. Before, those keys got a ghost hit and went straight back to the main head. Now they miss the ghost and re-enter the 102-slot small queue at freq 0. There, they have to collect `freq > 1` before small evicts them, or they are rejected again. On workload-a, enough of these medium-distance returners were real reuse that losing them cost more than the main evictions the change avoided.

## Takeaway

The hypothesis assumed the oldest ghost entries mostly cause low-value re-admissions. Workload-a shows the opposite: keys that return late in a `main_size`-length ghost window are still worth fast-tracking into main, and the small filter is too short to catch them on its own. Larger ghosts lost in earlier cycles (per the hypothesis rationale), and this smaller one also lost. So for this clear-on-pass SIEVE lineage, a shared ring of about `main_size` entries looks like a local optimum. Further gains are more likely to come from changing *which* keys are ghosted, or how a ghost hit is admitted, than from changing the ring length.
