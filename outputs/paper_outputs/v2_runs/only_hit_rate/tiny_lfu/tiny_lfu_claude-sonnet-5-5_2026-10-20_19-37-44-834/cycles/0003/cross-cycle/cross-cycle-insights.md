## Recurring wins

- Changes that preserve frequency information and only slightly loosen admission (cycle 1 window plus segmented main, cycle 3 tie-admit duel) are the only ones to win. The gains are marginal and near noise.
- Conservative-update sketches with wider counters (u8) cost no accuracy, so the sketch looks over-provisioned.

## Recurring failures

- Mechanisms that discard or hide frequency history lose hit-rate: aging, a larger window, a doorkeeper, 4-bit saturating counters with a shared hash. Each of these tested only on workload-a.
- Structural tuning of window size and admission on workload-a returns roughly nothing. Hit-rate is near its optimum on this stable, frequency-driven trace, and adaptive machinery such as hill-climbing has nothing to adapt to.
- Winner labels in a cohort of losers (cycle 2) are least-bad picks, not real gains.
- Fitness vectors have reported hit-rate only for three cycles in a row. Metadata-size and per-op-time, the stated goals of the compaction ideas, remain unmeasured.
- Infrastructure failures (a download rate limit in cycle 1) can reject an idea without testing it.

## Open questions

- Do the winners (windowed design, tie-admit duel) differ on metadata or per-op cost, or is any win real beyond noise on one trace and one cache size?
- Does aging or a doorkeeper pay off on a trace with shifting popularity? Does hill-climbing pay off on a trace whose best window size shifts?
- Can a compact sketch (4-bit counters) recover hit-rate with periodic halving and decorrelated row hashes? Do its size and speed savings justify any remaining loss?
- Does the cycle-1 u8 result generalize, or does it depend on conservative update?

## Per-policy notes

- TinyLFU, the sketch-gated admission baseline: already captures most of the available benefit on workload-a.
- Window plus segmented main (W-TinyLFU style): a small, unconfirmed gain. Window size is near its optimum, so further tuning is not worth doing.
- Tie-admit window-to-probation duel: the current base and the best-performing admission tweak so far. It is a very small gain.
- Sketch variants: u8 with single hash and conservative update is neutral. 4-bit packed with a shared hash loses hit-rate. A doorkeeper and aging both lose hit-rate on workload-a.
- Next steps: fix the fitness output to include metadata-size and per-op-time, add a second trace (shifting popularity, a different cache size), and cache the simulator's dependencies before running.
