## Recurring wins

- Only one cycle so far; nothing has yet repeated. Provisional signal: variants that target the single scored dimension (hit-rate) are the only ones that surface on the Pareto front.
- The baseline's arena-slot-reuse scaffolding and P1/P2/P3 invariants survive aggressive representational rewrites — large structural changes are cheap to attempt without breaking the gate.

## Recurring failures

- Bets whose intended payoff lives off the scored axes (per-op time, metadata size) are structurally sound but invisible to selection, so they get filtered out regardless of merit.
- Same-size deltas have not actually been demonstrated when a variant and the baseline are sampled at different cache sizes — apparent hit-rate movement is then unattributable.

## Open questions

- Can the fitness vector itself be expanded (per-op time, metadata size) so structural wins stop reading as neutral noise?
- Is the SLRU-style hit-rate movement a real scan-resistance effect, or a sampling artefact of mismatched size/trace pairings?
- Does CLOCK's small recency-fidelity loss matter on workloads that are less recency-skewed than the LIRS:multi1 trace?

## Per-policy notes

- LRU (baseline): structurally permissive — tolerated SLRU segmentation, CLOCK replacement, and u32+sentinel link reworks without invariant breakage.
- SLRU: showed qualitative scan-resistance, but same-size comparison against LRU was not actually established.
- CLOCK: only variant to reach the Pareto front on hit-rate; promising base for refinements (two-handed CLOCK, CLOCK-Pro-lite) that defend hit-rate while keeping the per-op-time advantage.
- u32+sentinel link representation: bet is purely on metadata size; not worth pursuing further until a metadata-size dimension is scored.
