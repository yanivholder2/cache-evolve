# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-05T23:33:37+00:00

## timeline

### 23:33:47 thinking



### 23:34:02 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Spending the *existing* long-term frequency signal more carefully beats adding new machinery: the first durable wins came from frequency-aware victim selection and from sharpening the count-min sketch, not from new policy mechanics.\n- Estimator fidelity was the single clearest early lever — widening the sketch lifted hit-rate when every structural change stalled — but it saturated quickly and stopped paying once cells separated the distinct-key set.\n- Strict, conservative guards consistently win over looser ones: the admission gate plus an accurate sketch keep the hot working set resident, so tighter selection rarely costs and often protects.\n- In late cycles the best outcomes are *ties* (hold-at-parent), which is itself the signal that the hit-rate axis is mined out rather than that a lever is working.\n\n## Recurring failures\n\n- Any mechanism that discards, resets, or withholds long-term frequency signal loses: sketch aging, an unconditional admission window, and the doorkeeper (in every form, isolated or not) all regressed.\n- Loosening or adding admission filtering keeps backfiring: extra gates, admission-side placement bypassing probation, and gate-loosening promotion contests all trade a proven-warm resident for a less-certain key. The probationary \"earn a hit\" filter does real work; weakening it costs hits.\n- Spending the signal more aggressively (deeper victim samples, larger probation, smaller protected tier) plateaus then reverses — the marginal structural effects are workload ceilings, not down-tuning misses.\n- Stacking two positive refinements does not compound: mechanisms that fire on the same rare \"head hides a hotter/colder key\" event overlap and net to nothing.\n- Aggressive memory narrowing (4x) is refuted — conservative update cannot hold accuracy at reduced width; the workload genuinely needs the cells.\n\n## Open questions\n\n- Is hit-rate now conclusively at its capacity ceiling? Evidence across cycles 0003–0008 is very strong: every SLRU steering point, both estimator axes (width and depth), and protected sizing are all probed and inert.\n- Where can saturation be turned into free efficiency? Depth was just shown redundant, so probe reducing depth and trimming protected slack for cost with no hit-rate loss.\n- Is there a *shallow* width reduction (e.g. 2x) that trades cells for cost before accuracy breaks, given 4x is refuted?\n\n## Per-policy notes\n\n- LRU/recency baseline: recency order already correlates tightly with sketch frequency on this trace, which is why frequency contests so rarely reverse an eviction or promotion.\n- TinyLFU estimator (count-min sketch): width was the durable lever early, now saturated; depth confirmed independently redundant; conservative update adds nothing on top of adequate width. Estimator fidelity is fully mapped.\n- W-TinyLFU window / doorkeeper / admission filters: refuted across multiple cycles — they bypass or reset signal the bare `incoming_freq > victim_freq` test already handles.\n- SLRU (protected/probationary): every steering point — admission, promotion entry, promotion gating, probationary eviction, protected demotion — is real but marginal or redundant; protected fraction favors *large* (>0.8 is a flat plateau), the smaller-fraction line is refuted.\n"
}
```

### 23:34:07 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 8-cycle ledger digest, using the four required sections in order.

Key cross-cycle synthesis:
- **Wins**: spending existing frequency signal carefully + early estimator-width fidelity; strict guards beat loose ones; late-cycle ties are themselves the ceiling signal.
- **Failures**: anything that discards/resets/withholds frequency signal (aging, window, doorkeeper); loosening admission; over-aggressive spending; non-compounding of overlapping refinements; 4x narrowing.
- **Open questions**: ceiling confirmation, and the efficiency pivot (trim now-redundant depth/protected slack, probe shallow 2x width).
- **Per-policy notes**: grouped under LRU baseline, TinyLFU estimator, W-TinyLFU/doorkeeper, and SLRU.


---

- finished: 2026-06-05T23:34:08+00:00
- duration: 30.23s
- status: ok
- input_tokens: 3
- output_tokens: 2107
- cache_read_tokens: 40527
- cache_write_tokens: 10555
- cost_usd: 0.088921
