# Cycle 0009 Insight

## Cohort outcome

Three variants were tried: a halved sketch aging threshold, a stricter admission margin (candidate must beat the victim by more than 1), and a CLOCK + conservative-update sketch gate replacing strict LRU. Only the stricter admission margin (v002) won, with a marginal hit-rate gain. The shorter aging window lost slightly. The CLOCK variant reported a hit-rate gain over strict LRU, but it did not win.

## Patterns

- Aging-window tuning is now bracketed from both sides. Longer (cycle 0007) and shorter (this cycle) both lost slightly, so the window sits at a flat optimum.
- The admission-margin win contradicts the earlier "admission gate is saturated" conclusion only weakly. The gain is small and may be noise. It suggests a stricter gate trades a few lost second-touch keys for less scan pollution, and counting rejected misses keeps that trade safe.
- CLOCK + sketch again shows a hit-rate edge over strict LRU without beating the segmented line. This is the third cycle with that pattern (0003, 0006, 0009).
- Metadata-size and per-op-time were again missing from the fitness vectors, now for nine cycles. The claimed cheaper CLOCK hits and the sketch costs remain unverified.
- Variant authors could not read the overlay source, so their interpretations rest on hypothesis text and fitness vectors only. All differences are marginal, on one trace at one cache size.

## Open questions

- Does the stricter margin hold on traces with more second-touch reuse or less scan pressure?
- Does CLOCK's cheaper hit path offset its miss-time sweep and sketch updates? That cannot be known without timing data.
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning the aging window and the admission margin; both are at a plateau on this trace. Treat v002's margin as a minor, optional tweak on the incumbent. Fix metadata-size and per-op-time measurement and add a second trace or cache size before generating more hypotheses, since the loop is selecting on marginal hit-rate edges alone. Then try a different axis, such as a new eviction signal (size or cost awareness), or a head-to-head cost-aware comparison of CLOCK + sketch against the segmented incumbent.
