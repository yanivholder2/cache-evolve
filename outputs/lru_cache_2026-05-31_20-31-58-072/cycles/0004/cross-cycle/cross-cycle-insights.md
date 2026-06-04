## Recurring wins

- Layering complementary mechanisms beats widening a single signal: the CLOCK-Pro-lite win (promotion class + ghost ring) came from pairing two distinct gaps being closed, not from deepening either one.
- Recency-encoding per-slot state pays off on the recency-skewed trace currently in use (Cold→Hot on second touch, evicted-key memory, ghost rings).
- Edits that touch a mechanism *already in the hot path* (e.g. ghost-ring length) are the only ones still moving the needle late in the run; mechanisms gated behind dormant code paths score at parity.

## Recurring failures

- Frequency-leaning per-slot state (saturating aging counters, GCLOCK) actively shields stale-popular keys on recency-skewed traces and regresses hit-rate.
- Pass-2 / Hot-side refinements (sticky demotion, Hot-ghosts, warm re-admit, Warm promotion ladders) at the current trace/capacity: pass 2 doesn't fire enough for these knobs to bite, so they score at exact parity.
- SLRU rebuilds have now twice failed to produce an interpretable result because variant and baseline samples were taken at disjoint capacities — a sampling-discipline failure, not a policy failure.
- Pure representation-only repacks (e.g. u32+sentinel) whose intended payoff lives on unscored axes (metadata size, per-op time) cannot be selected for and look like noise.
- Re-ranking the Cold victim pool (taint pre-pass, intra-Cold LRU): three orthogonal edits all bit-identical to parent — the spared slot dies a sweep or two later and the re-reference distribution doesn't fall in that window.

## Open questions

- Is the fitness vector itself the binding constraint? Per-op time and metadata size remain unscored after four cycles, so structurally meaningful variants keep scoring as noise.
- Has `lirs:multi1@512` been wrung out for this skeleton? Parity is now the dominant outcome, suggesting workload/capacity diversity may matter more than further policy creativity.
- Would the inert Hot-side and Cold-reordering knobs differentiate at smaller capacities (where pass 2 is forced to fire) or on traces with thicker re-reference tails?
- What pass-1 levers remain genuinely unpulled — Cold-admission gating *before* insertion, ghost-hit-targeted promotion seeding — as opposed to victim-pool reorderings that have now repeatedly tied?
- Can SLRU-style segmentation ever be evaluated honestly without first fixing matched-capacity sampling between parent and child?

## Per-policy notes

- **LRU (baseline):** structural scaffolding (arena-slot-reuse, P1/P2/P3 invariants) is permissive of large representational rewrites without breakage.
- **SLRU:** unresolved across two cycles purely because of sampling discipline; the mechanism itself is untested on a matched-size delta.
- **CLOCK (v002, cycle 1):** the original Pareto entrant; its win lived on per-op time, which is unscored, so subsequent cycles built on it for its hit-rate signal rather than its actual advantage.
- **GCLOCK / frequency-counter lineage:** dropped — regresses on the current recency-skewed trace.
- **CLOCK-Pro-lite (cycle 2 champion):** the lift comes from the *existence* of the ghost ring + promotion class. Internal-shape tuning across two follow-up cycles is now strongly diminishing.
- **Full-capacity ghost-ring CLOCK-Pro-lite (cycle 3 v001):** current working champion by marginal lift, with the ghost-sizing knob itself in deep diminishing returns — further size sweeps not worth a slot.
- **u32-sentinel / representation-only:** defer until a metadata-size dimension exists.
