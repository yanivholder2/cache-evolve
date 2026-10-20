# Cycle 0004 Insight

## Cohort outcome

All three variants composed pieces recommended by Cycle 0003 — a bounded promotion loop, an unconditional recency advance, and a two-tail eviction selector — with each producing a hit-rate lift but none clearing the Winners bar, primarily because per-op-time remains pinned at the benchmark's timer-resolution floor regardless of approach.

## Patterns

- **Three-swap cap resolves the single-swap bottleneck cleanly.** Extending the promotion budget to three swaps lets mid-bucket items reach the head zone without per-op-time cost, because the early-break on a failed swap means the common Zipfian hot-key case exits after one iteration. Cycle 0003's single-swap cap was the precise constraint; three swaps is sufficient to close the convergence gap.
- **Unconditional recency promotion beats frequency-gated promotion on this workload.** Removing the `lfu_counter` neighbour check and always advancing one step outperformed soft-LFU ordering: the frequency gate was stranding recently-reused items below cold neighbours, and always-advance keeps the tail genuinely cold for the eviction path.
- **The 32-byte aligned struct is a non-negotiable base.** Dropping the compact layout in v003 immediately inflated per-item footprint and likely reintroduced cache-line straddling, erasing the two-tail eviction benefit. Every future variant must carry the alignment fix forward explicitly.
- **The per-op-time measurement floor is now a visible ceiling.** Both v001 and v002 show no latency regression, but neither shows improvement; v002 explicitly attributes this to timer resolution. The benchmark's floor is masking whether any latency gains are real, which means further per-op-time optimisation hypotheses cannot be evaluated until timer resolution improves.

## Open questions

- Do the 3-swap loop (v001) and the always-advance rule (v002) address the same mid-bucket convergence bottleneck from different angles, or are they additive when composed?
- Is the two-tail eviction comparison (v003) worth re-testing in combination with the compact struct, and does it compose cleanly with either promotion strategy?
- Is the 100ns timer floor a benchmark configuration issue, or does meaningful latency differentiation require a workload with higher operation count / longer trace to amortise measurement noise?

## Continuation

The next cycle should compose the 32-byte aligned struct with the 3-swap bounded loop (v001) and test whether replacing the frequency gate with the unconditional always-advance rule (v002) produces additive hit-rate gains or converges to the same outcome. The two-tail eviction selector is worth a clean re-test paired with the compact layout. If per-op-time results remain flat after these compositions, the cycle after that should explicitly investigate whether the measurement floor is the blocker rather than introducing new structural changes — resolving this ambiguity is now more urgent than adding further promotion complexity.
