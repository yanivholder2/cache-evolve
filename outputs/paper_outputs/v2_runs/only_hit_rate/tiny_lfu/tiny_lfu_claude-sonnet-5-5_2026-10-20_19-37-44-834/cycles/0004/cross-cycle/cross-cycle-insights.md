## Recurring wins

- Changes that preserve frequency information and only loosen admission slightly (cycle 3 tie-admit duel) give tiny gains; this is the only kind of change that has helped.
- Changes that leave frequency estimates intact (segment split, single-hash row derivation) are neutral on hit-rate, so they are safe candidates for cost wins.
- Winners are repeatedly ties or marginal gains near noise (cycles 1-4), not clear improvements.

## Recurring failures

- Anything that discards or hides frequency information lost hit-rate: aging/decay, larger window, doorkeeper, 4-bit saturating counters, narrower sketch rows.
- Hit-rate-only tuning of window size, protected-segment share and admission on workload-a returns roughly nothing. The workload looks stable, frequency-driven and saturated.
- Adaptive window tuning has no way to show value on a single stable trace.
- Evaluation gap: for four cycles the fitness vectors have reported hit-rate only. Metadata-size and per-op-time, the stated goals of the cost-oriented ideas, remain unmeasured, so no cost trade has been judged.
- Environmental failures (cycle 1 download rate limit) can block an idea without testing it. Variant authors often cannot read overlay source, so explanations are plausible and not confirmed.

## Open questions

- Fix the evaluation first. Add metadata-size and per-op-time to the fitness output, plus a second trace (shifting popularity or a different cache size).
- Does aging pay off on a trace with real popularity shifts or a longer sample period?
- Does a doorkeeper help with the admission +1 offset adjusted, or under real sketch pollution?
- Does single-hash row derivation actually cut per-op-time?
- Would a gentler width reduction than 2x keep hit-rate while saving size?
- Can a compact 4-bit sketch recover with aging and decorrelated row indices, and do its savings justify any loss?
- Are the small wins (tie-admit duel) real or noise?

## Per-policy notes

- TinyLFU, windowed with segmented main (current base includes the tie-admit duel): the baseline's sketch-gated admission already captures most of the benefit on workload-a.
- Sketch: u8 counters with conservative update lost no accuracy. 4-bit counters with a shared hash lost hit-rate, and halving the row width lost hit-rate. Single 64-bit hash indexing held hit-rate level and is a candidate cost win to verify on speed.
- Window: both larger and adaptively tuned windows were neutral or negative.
- Protected/probation split: irrelevant at this cache size, so drop further sizing work.
- Aging and doorkeeper: both lost on workload-a. They are untested on drifting workloads.
