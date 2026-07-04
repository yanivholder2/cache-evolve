# Cycle 0006 Insight

## Cohort outcome

The cohort probed whether the accumulated mechanism stack could be reformed: CLOCK replaced bulk-decay eviction, selective decay targeted only minimum-counter items, and a clean-slate pure-LFU probe tested wide geometry without any positional tricks. All three fell short of the prior best; selective decay (v002) was selected as the least-disruptive departure.

## Patterns

- **Positional recency in the accumulated stack is non-redundant.** Both CLOCK and selective decay improved frequency discrimination but lost the recency signal embedded in bucket position — the LRU-front (index-0) tie-breaker implicitly encodes insertion age and rewards workload-a's temporal locality. Removing that signal cost more than better frequency ordering gained in either case.
- **Episodic bulk decay outperforms per-item incremental aging on this workload.** CLOCK's sweep decrements counters on every eviction call, steadily eroding active items' protection. The bulk-decay fallback fires rarely, concentrating its counter-compression cost; CLOCK trades that infrequent cost for a steady one that makes still-relevant items evictable sooner.
- **Pure min-counter LFU on wide buckets is a near-approximation of the full stack.** Wide geometry plus frequency-only eviction closed most of the gap from the unmodified baseline to the accumulated best-known, confirming that a large candidate pool largely substitutes for cold-first bias and decay. The remaining shortfall quantifies the combined contribution of those positional mechanisms — small but real.
- **The local optimum is increasingly confirmed as robust.** Three consecutive cycles of structural challenges have each returned regressions. The accumulated stack's lead is not narrow-margin luck; it reflects genuine complementarity between its components.

## Open questions

- Is the recency benefit of the LRU-front tie-breaker workload-specific, or would it persist on traces with weaker temporal locality?
- Per-op latency remains unmeasured: does the simpler pure-LFU wide-bucket path offer a better hit-rate/latency trade-off than the full stack, even at a small hit-rate penalty?
- Could a hybrid — LFU eviction with the decay trigger retained but selective-decay semantics applied — recover the recency loss while preserving counter differentiation?

## Continuation

The accumulated stack continues to hold despite sustained pressure. The closest alternative is v003's pure-LFU + wide-geometry configuration, which approaches but does not match the best-known. Next cycle should pivot to latency measurement: compare the full accumulated stack against v003's simpler path to determine whether the hit-rate delta is worth the mechanism complexity. If those results show acceptable parity, the simpler path becomes the preferred baseline for subsequent exploration. Do not revisit CLOCK or selective decay — both confirm that disrupting positional recency costs more than it gains on this workload. An adaptive two-queue or ARC-style policy that separates recency and frequency into explicit queues (rather than encoding both in position) remains the most promising structural alternative if the latency comparison does not resolve the choice.
