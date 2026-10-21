# Variant v001 Insight

## Helped

- hit-rate: The `else` branch in `put` now moves a victim that wins its contest to probation MRU, so later contests sample a new 8-entry window. On workload-a this gave at most a noise-level gain, so the frozen-window problem the hypothesis targeted was not costing real hits.

## Hurt

## Takeaway

Rotating the surviving victim does not lower the admission bar the way the hypothesis expected. The victim that survives is, by construction, the lowest-estimate entry in the 8-entry window. Rotating it out swaps in the next-oldest probation entry, so the window's minimum stays the same or rises unless that next entry happens to be colder. The rotated key also comes back to the head later with an equal or higher count. Rotation mostly changes *which* resident is evicted once a newcomer wins. It barely changes *how often* newcomers win, and with non-aging lifetime counts the old probation entries all carry similar estimates. This variant joins cycles 0003 and 0009 as evidence that victim-window mechanics (depth, and now rotation) are inert on this trace. The admission contest needs a different comparison signal, such as count aging or decay, rather than different candidates.
