# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:56:02+00:00

## timeline

### 19:56:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The second-hit admission filter is the one real, repeatedly confirmed win (cycles 0001, 0002, 0004). It screens one-hit-wonders, skips first-hit remove-and-promote work, and was independently re-derived from baseline with the same benefit.\n- Packing the filter flag into spare bits keeps metadata at baseline item size while keeping the hit-rate and latency benefit (cycle 0002).\n- Subtractive changes pay off: removing bookkeeping the policy never reads (per-get stats, timestamp writes) improved latency (cycle 0004).\n\n## Recurring failures\n\n- Position-only levers (uniform MRU, cold/hot insert, cold demotion) gave small, mixed or noise-level gains. Stacking them on the filter overlaps, because they target the same scan resistance, and trades hit-rate against latency.\n- Richer admission or recency signals (multi-bit counter, ref bit) added almost no hit-rate on top of the filter and cost latency. Hit-rate looks saturated on this trace.\n- Adding hot-path mechanisms (guards, shortcuts, atomics, early returns, free-slot fast paths, merged scans) consistently lost latency. Branches that only sometimes succeed are not free, and buckets are mostly full.\n- \"Winner\" status is weak evidence: many gains are within trace or run noise, and regressions sometimes appeared on untouched paths (layout or measurement noise).\n\n## Open questions\n\n- Which latency deltas are real versus run-to-run or code-layout noise? Re-measure with repeats.\n- Where do get and put time actually go (lookup, locking, bucket scan)? Nothing has measured this.\n- How much of the cycle 0004 v003 gain comes from dropping dead bookkeeping versus the filter itself?\n- Does the cleaned-up minimal filter match or beat the cycle 0002 v001 incumbent?\n- Do conclusions hold on a different trace (scans, frequency skew) rather than the recency-skewed LIRS trace?\n\n## Per-policy notes\n\n- DASH segment with the bit-packed second-hit filter: current best base. Use the cycle 0004 v003 minimal version, compared against cycle 0002 v001.\n- Promotion and insert-position variants (MRU, tiered, cold first admission, cold demotion): considered exhausted. Do not revisit without a latency re-measurement.\n- Eviction-order variants (CLOCK-style second chance): no hit-rate benefit, extra scan cost.\n- Next direction: subtractive latency work and a different evaluation trace, not new policy levers.\n"
}
```

### 19:56:18 chat

I wrote `cross-cycle-insights.md` to the cycle 0004 output dir, with the four required sections in the fixed order.

- **Recurring wins:** The second-hit admission filter is the one confirmed win across cycles 0001, 0002 and 0004. Packing its flag into spare bits keeps the item at baseline size. Removing bookkeeping the policy never reads also helped latency.
- **Recurring failures:** Insert-position levers gave small or mixed gains and overlap when stacked. Richer admission or recency signals added almost no hit-rate. Hot-path mechanisms (guards, shortcuts, atomics, fast paths) kept costing latency. "Winner" status is weak evidence, since many gains are within noise.
- **Open questions:**
  - Are the latency deltas real or run-to-run noise?
  - Where do get and put time actually go?
  - How much of cycle 0004 v003's gain comes from dropping dead bookkeeping rather than the filter?
  - Does v003 match or beat the cycle 0002 v001 incumbent?
  - Do the conclusions hold on a different trace?
- **Per-policy notes:** The bit-packed second-hit filter in the DASH segment is the base to build on. Position variants and eviction-order variants are marked as exhausted. The next direction is subtractive latency work and a different evaluation trace.

I used only qualitative terms and invented no numbers.


---

- finished: 2026-10-20T19:56:18+00:00
- duration: 16.63s
- status: ok
- input_tokens: 4
- output_tokens: 1476
- cache_read_tokens: 36599
- cache_write_tokens: 8146
- cost_usd: 0.051443
