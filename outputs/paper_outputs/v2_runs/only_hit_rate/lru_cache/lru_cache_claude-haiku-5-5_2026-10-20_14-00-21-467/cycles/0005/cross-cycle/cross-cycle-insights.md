## Recurring wins

- Eviction-neutral representation and hit-path changes (sentinel index, head-hit relink skip, pre-sized key index, hit-path lookup collapse) have repeatedly held hit-rate exactly equal to their parent. They are safe to carry forward, but no cycle has yet shown a measured size or per-op gain from them.
- Changing where new keys enter the recency order (inserting at the LRU tail with periodic MRU promotion) is the only hit-rate gain in the ledger so far. It adds no new structure, which makes it the cheapest route to scan resistance seen so far. It has been observed once, on one trace.

## Recurring failures

- Changes that alter eviction order with extra structure have lowered hit-rate on workload-a at size 2048: the probation/protected split and the referenced-bit second-chance approximation both dipped below parent. Strict recency ordering appears to be doing useful work on this trace.
- Ghost-based admission filtering has produced no measurable effect in two cycles (0002, 0003). It may never have engaged on this trace, so the null result is not yet informative.
- Winner status is being assigned without a visible basis in the evidence. Variants with hit-rate equal to or below parent, and with no per-op or size data, have been named Winners in cycles 0003, 0004, and 0005. This selection-criterion problem recurs and undermines lineage decisions.
- Size and per-op goals have never been scored. Every cycle's changes were motivated by metadata or time, yet none has recorded those dimensions. This gap spans all five cycles.
- Several cycles lost variants to infrastructure failures (session limit, missing insight file). Those variants' strategies are untested, not refuted.

## Open questions

- Do the size and per-op costs or gains of any eviction-neutral change show up once measured? Until they do, the layout and hit-path lineages have no evidence on their stated goals.
- Does the LRU-tail insertion gain hold on other traces and cache sizes, or is it specific to workload-a at size 2048? Its benefit should concentrate on scan-heavy traffic, which has not been tested.
- Is the segmented split dead as written, or does its protected-cap and demotion rule carry headroom? The cycle 0004 hit-rate direction for that lineage is still unreported.
- Which parent do the reported hit-rate deltas refer to? Lineage has been ambiguous in cycles 0002 and 0004, which blocks clean comparison.
- Did the ghost admission path ever run on the trace? If not, which workload would exercise it?

## Per-policy notes

- Strict LRU (move-to-front): the strongest signal so far is that its ordering should be kept. Neutral layout and hit-path changes preserve it, while second-chance and segmented variants that loosen it lowered hit-rate.
- LRU with insertion-position change: the only hit-rate gain seen. Treat as the lead candidate, pending generalisation checks.
- Segmented (probation/protected) recency: lowered hit-rate in its one reported cycle. Its extra list pointer and demotion logic likely add per-op and size cost that has not been measured.
- Second-chance / referenced-bit: lowered hit-rate and was still listed as a Winner. Do not extend without re-checking that selection.
- Ghost-based admission filter: no measured effect so far; engagement on the trace is unverified. Keep only as a hit-rate bet if it can be shown to fire.
