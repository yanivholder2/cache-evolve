# Variant v001 Insight

## Helped

(none — no dimension improved)

## Hurt

- hit-rate: Halving every counter each 10×capacity increments erases the long-run frequency gap that the strict `incoming_freq > victim_freq` rule relies on. Under this trace, stable hot keys lose their advantage over one-hit scan/cold keys, so admission turns noisier and the cache admits or retains worse entries. The "stale hot keys block newcomers" premise did not hold for workload-a.
- per-op-time: `increment` now does extra bookkeeping on every get and put, and `reset()` sweeps all 4×4096 counters in a burst. That shows up in the p99 tail and also lifts the median.
- metadata-size: The sketch table is unchanged, but the two new `usize` fields on `CountMinSketch` (`additions`, `sample_size`) enlarge the `TinyLfuCache` struct. The headline node-size summary is unaffected.

## Takeaway

Adding TinyLFU-style aging (halve at sample_size = 10×capacity) with the baseline's strict-greater admission and no doorkeeper or window made the cache worse and slower. The baseline's never-decaying u32 counters were an asset on this workload, not a liability. Aging only pays off if the sample period or admission rule is tuned to the trace's frequency drift. A more conservative period, or aging combined with other changes, would need to be tested separately.
