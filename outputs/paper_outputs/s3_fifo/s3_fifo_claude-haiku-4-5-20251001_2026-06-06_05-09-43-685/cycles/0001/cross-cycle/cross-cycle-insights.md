# Cross-Cycle Insights

## Recurring wins

- **Tighter admission filtering + queue resizing**: All three Cycle 0001 winners (v001, v002, v003) achieved 28–66% latency improvements by constraining main-queue population through stricter promotion thresholds or explicit queue sizing. The consistency across variants suggests reduced contention and lookup-chain overhead is a reliable latency lever.
- **Latency-first optimization is viable**: The cohort demonstrates that prioritizing operation latency over hit-rate is a sound strategy; all three variants succeeded despite hit-rate trade-offs.

## Recurring failures

- **Loose admission filtering**: Cycle 0001 strongly signals that loosening admission gates backfires. The cohort convergence on tighter filtering as the winning approach implies that greedy retention of moderately-accessed items increases main-queue contention and lookup cost, offsetting any hit-rate gain.

## Open questions

- Can frequency-decay or recency-weighted heuristics recover hit-rate without losing the latency gains from tighter filtering?
- Is the latency-hit-rate frontier workload-dependent, or are the trade-offs universal across access patterns?
- Is there a tuning sweet spot (e.g., modest latency sacrifice for meaningful hit-rate recovery) or is the frontier binary?
- How sensitive are these variants to promotion-threshold and queue-sizing parameters across different workload phases?

## Per-policy notes

- **Strict promotion thresholds (v001, v002)**: Effective at reducing main-queue bloat and latency. v001 shows the best balance with marginal miss impact; v002 uses ghost-hit re-routing to mitigate hit-rate loss but incurs 8%+ decline. Both block moderately-accessed items.
- **Explicit queue resizing (v003)**: Achieves latency gains despite hit-rate decline, suggesting the workload benefits from smaller resident set and reduced eviction overhead. Implies admission filters are not the only lever for contention reduction.
- **Hit-rate as dependent variable**: All winners accept hit-rate sacrifice as the cost of latency optimization. The pattern suggests hit-rate recovery must come from alternative levers (e.g., adaptive thresholds, frequency decay) rather than looser filters.
