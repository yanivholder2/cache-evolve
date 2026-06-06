# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-06-05T23:25:37+00:00

## timeline

### 23:25:55 thinking



### 23:26:05 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Spending the *existing* long-term frequency signal more carefully beats\n  altering policy mechanics: the first clean gain came from frequency-ranked\n  victim selection, and the most durable early lever was sharpening the\n  estimator (wider count-min sketch) while leaving every mechanic untouched.\n- Estimator fidelity was the single clearest lift on this trace — buying a\n  more accurate count won where redesigning the signal's use lost.\n- In later cycles, \"wins\" are won by *not regressing*: every structural lever\n  (SLRU split, promotion gating, demotion choice) is directionally correct but\n  clears the noise floor by only a hair.\n\n## Recurring failures\n\n- Any mechanism that discards, resets, ages, or withholds long-term frequency\n  signal loses: sketch aging, unconditional admission window, doorkeeper.\n- Extra admission filtering keeps failing — the bare `incoming_freq >\n  victim_freq` gate plus a sharp sketch already screens one-hit wonders, so\n  doorkeepers, entry floors, and admission-side placement only delay warming\n  keys (or find nothing to gate) and cost hit-rate.\n- Spending the signal more aggressively backfires: deeper victim samples\n  plateau then reverse; stacking independent levers overlaps on the same rare\n  cases and nets nothing.\n- Aggressive memory narrowing (4x) breaks accuracy — the workload genuinely\n  needs more than capacity×1 cells; conservative update does not rescue it.\n\n## Open questions\n\n- Is hit-rate at this capacity conclusively capped? Evidence across cycles\n  0003–0007 is very strong — even a correctly-firing entry floor gates nothing.\n- On the efficiency/cost axis (now the live frontier): does a *shallow* width\n  reduction (e.g. 2x) hold accuracy where 4x failed?\n- Are the marginal structural levers a workload ceiling rather than a tuning\n  miss — would they pay off under a churnier or more skewed trace?\n\n## Per-policy notes\n\n- **LRU baseline / victim choice:** tail rarely hides a much-colder item;\n  limited tail churn caps any victim-side change at a slim margin.\n- **Sketch aging / admission window (W-TinyLFU):** inert on this\n  popularity-stable, low-churn trace — they assume shifting popularity or burst\n  protection that this workload does not exhibit.\n- **Count-min sketch:** width is the key fidelity lever but plateaus once cells\n  separate distinct keys; conservative update is redundant with a wide sketch\n  and cannot hold accuracy at reduced width.\n- **SLRU:** favors a *larger* protected tier (smaller-fraction line refuted);\n  every steering point — admission, promotion entry, promotion gating,\n  probationary eviction, protected demotion — has been mapped and each is\n  marginal or redundant because recency order already tracks sketch frequency.\n"
}
```

### 23:26:10 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 7-cycle ledger. Summary of what the four sections capture:

- **Recurring wins:** Carefully spending existing frequency signal (frequency-ranked victim choice) and sharpening the estimator (wider sketch) were the only clean lifts; later cycles win mainly by not regressing.
- **Recurring failures:** Anything that discards/resets frequency signal (aging, window, doorkeeper), extra admission filtering, over-aggressive signal spending, and aggressive sketch narrowing.
- **Open questions:** Whether hit-rate is conclusively capped, whether a shallow 2x width cut holds accuracy on the efficiency axis, and whether marginal structural levers are a ceiling vs. a tuning miss.
- **Per-policy notes:** Grouped observations for LRU/victim choice, sketch aging/window, the count-min sketch, and SLRU.


---

- finished: 2026-06-05T23:26:10+00:00
- duration: 33.41s
- status: ok
- input_tokens: 3
- output_tokens: 2353
- cache_read_tokens: 39460
- cache_write_tokens: 9718
- cost_usd: 0.088356
