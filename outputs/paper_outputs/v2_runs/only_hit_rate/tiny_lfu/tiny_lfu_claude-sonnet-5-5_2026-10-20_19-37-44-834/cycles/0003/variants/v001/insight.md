# Variant v001 Insight

## Helped

- hit-rate: Hill-climbing `window_cap` from the 1% start gives a marginal gain. The tuner apparently stays close to the fixed-window optimum and picks up a little extra when a sample period favors a slightly different window size. The gain is within noise for a single trace.

## Hurt

- hit-rate: No dimension clearly regressed. The only cost is the unmeasured extra machinery, since the fitness vector tracks hit-rate alone. That machinery is per-period sampling and the segment resizing and demotions when the window moves. Exploratory steps may also cost some hits in the periods where they move away from the optimum.

## Takeaway

On this stable workload the 1% window is already near-optimal, so online hill-climbing roughly reproduces the parent's behavior instead of improving it. The adaptive window pays off only on traces whose best window size shifts, and a single stable trace cannot show that. Judge the idea on a multi-trace or phase-changing benchmark before keeping its added complexity.
