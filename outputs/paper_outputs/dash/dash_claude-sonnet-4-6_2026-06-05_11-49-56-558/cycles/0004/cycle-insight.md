# Cycle 0004 Insight

## Cohort outcome

All three variants probed structural axes around stash admission and promotion gating — ghost-based bypass, LFU-gated promotion, and stash-free direct insertion — in line with Cycle 0003's call to explore unexplored terrain. Only v002 (ghost re-admission) won, and only by a margin that is effectively noise; the other two variants regressed.

## Patterns

- **The stash is a load-bearing quality gate, not just a capacity buffer.** Both attempts to weaken or circumvent it — front-door bypass on cold insertion (v003) and threshold-gating the hot-item promotion pipeline (v001) — degraded hit-rate. Removing or slowing stash traversal allows cold or slow-to-qualify items to displace established warm residents, which costs more than the saved hop recovers. This role as admission filter appears fundamental for workload-a's stable hot set.
- **Ghost re-admission is safe but near-neutral.** The ghost set (v002) produced only a marginal, noise-level improvement: the workload's reuse pattern does not cycle through the stash boundary frequently enough for bypass tracking to yield a real benefit. The mechanism is correct in principle but mismatched to this workload's access characteristics.
- **Promotion gating introduces friction with no real payoff.** The LFU threshold in v001 was designed to filter one-time scans from the probing tier, but scan pollution was not a meaningful loss source here — probing slots were already being used productively. Adding a counter-gated delay simply slows the hot-item pipeline.

## Open questions

- With topology, re-entry, admission control, and promotion gating all explored or exhausted, what genuinely novel levers remain for workload-a? Scan-resistant insertion *within* the normal tier (rather than at the stash boundary) has not been probed.
- Would any of these admission-control ideas produce real gains on a workload with heavier scan or bursty cold-item arrival patterns, where the quality-gate role of the stash is under more stress?
- Is the ghost set's near-zero gain truly a ceiling, or does it depend on the ghost set size and Front-insertion position chosen for re-admitted items?

## Continuation

The stash-as-admission-filter finding should be treated as a constraint: next-cycle hypotheses must not weaken or bypass stash traversal on new insertions. The ghost set (v002) can be set aside — it is safe but contributes nothing to workload-a. The improvement space for this workload is now quite narrow under current structural axes; the next hypotheses-writer should consider either intra-normal-tier eviction ordering (e.g., scan resistance or segmented LRU within the hot tier), or shifting to a second workload trace to reveal whether a different access pattern unlocks frequency-based or admission-control gains that are structurally blocked on workload-a.
