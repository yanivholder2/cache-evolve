# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-05T23:19:04+00:00

## timeline

### 23:19:19 thinking



### 23:19:19 chat

I'll write the cross-cycle insights as a fresh projection of the six-cycle ledger.

### 23:19:27 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Buying a sharper estimator was the single cleanest lift on this workload:\n  widening the count-min sketch raised hit-rate where no policy-mechanic\n  change could. It was the one durable upstream lever.\n- Spending the *existing* frequency signal carefully — ranking a few tail\n  candidates by estimate — won the first cycle, where mechanisms that\n  discarded signal lost. Careful selection beat aggressive selection.\n- Structural levers (frequency-ranked victim sampling, SLRU\n  recency-protection, frequency-gated promotion, protected-tier demotion)\n  are consistently *directionally* correct and avoid regressions, even when\n  their lift is at the noise floor.\n\n## Recurring failures\n\n- Any mechanism that resets, withholds, or bypasses long-term frequency\n  signal loses: aggressive sketch aging, an unconditional admission window,\n  and doorkeepers all cost hit-rate. This is the most robust negative result.\n- Extra admission filtering is refuted across multiple cycles. The bare\n  `incoming_freq > victim_freq` gate already screens one-hit-wonders, so\n  added doorkeepers only delay warming keys.\n- Spending the signal more aggressively backfires (deeper victim samples,\n  shrinking the protected fraction) — it tramples recency-warm or\n  warming-hot keys before counts accumulate.\n- Pushing an already-exhausted lever adds nothing: further sketch width and\n  conservative update proved redundant with each other once cells separated\n  the distinct-key set.\n- Compounding overlapping mechanisms does not stack — victim sampling and\n  promotion-gating fire on the same rare case and net to roughly nothing.\n- Null results have twice traced to mechanism bugs (an unsatisfiable entry\n  floor); verify a lever actually engages before reading it as workload\n  evidence.\n\n## Open questions\n\n- Is hit-rate at this capacity now conclusively capped? Evidence across the\n  later cycles strongly suggests a ceiling, with every structural placement\n  mapped and each marginal.\n- Could a *narrower* sketch paired with conservative update hold accuracy at\n  lower memory cost — trading now-wasted width for contention CU can correct?\n- Would the marginal structural levers ever pay off materially under a\n  churnier / more skewed workload, or is their marginality intrinsic here?\n- When should the search decisively pivot from hit-rate to efficiency/cost\n  dimensions instead of further structural refinement?\n\n## Per-policy notes\n\n- **Workload signature:** popularity-stable, low-churn. Mechanisms assuming\n  shifting popularity (aging) or burst protection (window) find no problem to\n  solve and only add noise. A lever matters only when the LRU head hides a\n  key much colder or hotter than recency order implies — a rare event.\n- **Estimator (count-min sketch):** the productive axis, but now closed.\n  Width lift plateaus once cells separate distinct keys; width and\n  conservative update are redundant, not additive (confirmed by direct test).\n- **Admission gate:** the `incoming_freq > victim_freq` test plus a sharp\n  sketch already keep the hot set resident, which caps every downstream\n  structural lever. Added admission filtering / doorkeepers refuted.\n- **SLRU:** comprehensively mapped — admission, promotion entry, promotion\n  gating, probationary eviction, and protected demotion all probed. Each is\n  real but marginal or redundant. The protected tier should stay *large*;\n  shrinking it is refuted. Recency order within both segments already tracks\n  sketch frequency closely, so frequency contests rarely reverse a decision.\n"
}
```

### 23:19:32 chat

Wrote `cross-cycle-insights.md` as a fresh projection of all six ledger cycles.

Key synthesis across the run:
- **Wins:** estimator sharpening (sketch width) was the one durable lever; careful spending of existing frequency signal beats aggressive spending; structural levers are directionally right but marginal.
- **Failures:** anything that resets/withholds long-term frequency signal loses (aging, windows, doorkeepers); extra admission filtering refuted repeatedly; overlapping mechanisms don't stack; pushing exhausted levers (more width, CU) adds nothing.
- **Open threads:** is hit-rate ceiling-capped, can a narrower sketch + CU trade accuracy for memory, and when to pivot to efficiency/cost dimensions.
- **Per-policy:** documented the stable low-churn workload signature, the now-closed estimator axis, the dominant admission gate, and the comprehensively-mapped SLRU.


---

- finished: 2026-06-05T23:19:32+00:00
- duration: 28.26s
- status: ok
- input_tokens: 3
- output_tokens: 2090
- cache_read_tokens: 38542
- cache_write_tokens: 8534
- cost_usd: 0.07906
