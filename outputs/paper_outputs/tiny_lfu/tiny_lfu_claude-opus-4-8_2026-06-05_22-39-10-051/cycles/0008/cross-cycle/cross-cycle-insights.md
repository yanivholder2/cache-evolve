# Cross-Cycle Insights

## Recurring wins

- Spending the *existing* long-term frequency signal more carefully beats adding new machinery: the first durable wins came from frequency-aware victim selection and from sharpening the count-min sketch, not from new policy mechanics.
- Estimator fidelity was the single clearest early lever — widening the sketch lifted hit-rate when every structural change stalled — but it saturated quickly and stopped paying once cells separated the distinct-key set.
- Strict, conservative guards consistently win over looser ones: the admission gate plus an accurate sketch keep the hot working set resident, so tighter selection rarely costs and often protects.
- In late cycles the best outcomes are *ties* (hold-at-parent), which is itself the signal that the hit-rate axis is mined out rather than that a lever is working.

## Recurring failures

- Any mechanism that discards, resets, or withholds long-term frequency signal loses: sketch aging, an unconditional admission window, and the doorkeeper (in every form, isolated or not) all regressed.
- Loosening or adding admission filtering keeps backfiring: extra gates, admission-side placement bypassing probation, and gate-loosening promotion contests all trade a proven-warm resident for a less-certain key. The probationary "earn a hit" filter does real work; weakening it costs hits.
- Spending the signal more aggressively (deeper victim samples, larger probation, smaller protected tier) plateaus then reverses — the marginal structural effects are workload ceilings, not down-tuning misses.
- Stacking two positive refinements does not compound: mechanisms that fire on the same rare "head hides a hotter/colder key" event overlap and net to nothing.
- Aggressive memory narrowing (4x) is refuted — conservative update cannot hold accuracy at reduced width; the workload genuinely needs the cells.

## Open questions

- Is hit-rate now conclusively at its capacity ceiling? Evidence across cycles 0003–0008 is very strong: every SLRU steering point, both estimator axes (width and depth), and protected sizing are all probed and inert.
- Where can saturation be turned into free efficiency? Depth was just shown redundant, so probe reducing depth and trimming protected slack for cost with no hit-rate loss.
- Is there a *shallow* width reduction (e.g. 2x) that trades cells for cost before accuracy breaks, given 4x is refuted?

## Per-policy notes

- LRU/recency baseline: recency order already correlates tightly with sketch frequency on this trace, which is why frequency contests so rarely reverse an eviction or promotion.
- TinyLFU estimator (count-min sketch): width was the durable lever early, now saturated; depth confirmed independently redundant; conservative update adds nothing on top of adequate width. Estimator fidelity is fully mapped.
- W-TinyLFU window / doorkeeper / admission filters: refuted across multiple cycles — they bypass or reset signal the bare `incoming_freq > victim_freq` test already handles.
- SLRU (protected/probationary): every steering point — admission, promotion entry, promotion gating, probationary eviction, protected demotion — is real but marginal or redundant; protected fraction favors *large* (>0.8 is a flat plateau), the smaller-fraction line is refuted.
