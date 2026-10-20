# Cycle 0010 Insight

## Cohort outcome

The cohort tried choosing the probationary victim by shared frequency (v001, the recorded Winner), periodic floor-halving of shared counters (v002), and a ghost-list override that admits recurring keys without a counter check (v003). Only v001 reports a hit-rate gain on workload-a, and it is marginal and from a single trace and size. v002 and v003 both lost hit-rate.

## Patterns

- The Winner's evidence matches its label this cycle: v001 is the only variant with a reported gain, unlike Cycles 0004 and 0006 through 0009. The gain is small and single-trace, so it is a candidate, not a result.
- Both losers fail on the admission path. v002 floors counts so one-hit residents drop to zero, and the `max(1)` floor on incoming keys then lets one-shot keys beat them. v003 lets recurrence bypass the `incoming > victim` comparison. In both, a loosened comparison admits keys the strict counter ordering would have kept out, and hit-rate falls.
- Aging is still negative. Cycle 0009 found that zero-pruning hurt; v002 shows plain floor-halving without removal also hurts, so the removal was not the only cause. Halving at small magnitudes erases the one-hit versus no-hit distinction the gate relies on.
- The ghost-list result differs from Cycle 0007's recurrence gain. There the ghost list refused first-time inserts; here it overrides a counter rejection. The mechanism is different, so the Cycle 0007 gain is not contradicted, but the recurrence signal has not yet helped when it can skip the counter check.
- Cost is unmeasured again. v001 adds a probationary scan on every miss, v003 writes to the ghost list on every bypass, and v002 runs a full halving pass. None of the three reports metadata size or per-op time, and the fitness vector still lacks these dimensions.

## Open questions

- Does v001's marginal gain hold on a second trace and a second cache size?
- Does the extra probationary scan per miss cost enough per-op time to offset a small hit-rate gain?
- Does v003 lose because the counter check is skipped, or because ghost-list writes from one-shot keys push out useful ghosts? The variant insight proposes separating these.
- Does a floor of 1 for previously seen keys (`max(1, v / 2)`) keep one-hit history without the admission flip seen in v002?

## Continuation

Next cycle should keep shared-frequency victim selection (v001) as the base, and confirm it on a second trace and size with per-op cost in the fitness vector. Drop the ghost override that skips the counter comparison, but keep recurrence as a signal to retest with the counter check kept on the ghost path, since Cycle 0007's gain is still open. Do not pursue plain halving or zero-pruning; if aging is retested, keep previously seen counts at or above 1. Note that Cycle 0009's advice to halve without removing entries is incomplete, since plain floor-halving also lost hit-rate here.
