# Variant v003 Insight

## Helped

- hit-rate: Gating stash→target promotion behind a 2-hit `lfu_counter` threshold turns the stash into a frequency admission filter, so one-hit-wonder keys age out of the stash without ever evicting an already-promoted hot key from a normal bucket — the protected working set survives longer and hit rate rises.

## Hurt

- (none observed on the measured dimension)

## Takeaway

Reusing the existing per-item `lfu_counter` as an in-place stash hit counter, and only promoting once it reaches `STASH_PROMOTE_THRESHOLD = 2`, is enough to convert the stash from a pass-through staging tier into a frequency gate that protects warm items — a clear win at threshold 2, with the threshold itself an unexplored knob a later cycle can sweep.
