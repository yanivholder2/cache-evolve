# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:46:56+00:00

## timeline

### 19:47:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_19-33-45-727/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission-side changes (second-hit gate) pay off more reliably than insert-position changes; they skip remove-and-promote work on first hits.\n- Metadata-free implementation matters: bit-packing the admission flag into spare bits kept the hit-rate benefit, restored baseline item size, and improved latency (cycle 2 v001 is the current best base).\n- Scan resistance (keeping one-hit-wonders out of hot tiers) is the theme behind every gain so far.\n\n## Recurring failures\n\n- Stacking position levers (cold first admission, cold demotion, flag reset) on top of admission or uniform MRU: levers overlap rather than add, giving weak or mixed results with a hit-rate/latency trade-off.\n- Position-only tweaks: small gains, no metadata or work change, latency shifts likely noise. Looks near exhausted as a lever.\n- Unmeasured latency claims: several latency changes were explained only by inference, so they are not trustworthy.\n\n## Open questions\n\n- Can the spare bits in the packed flag hold a small hit counter or an age/recency bit, giving richer admission signals at zero metadata cost?\n- Does the packed filter keep its latency gain with a milder demotion penalty (cold demotion without flag reset)?\n- Are the small hit-rate gains (e.g. cold first admission on uniform MRU) real or within trace noise? Needs latency/hit-rate re-measurement.\n- Does uniform MRU's gain come from promotion or from demotion cascades (never separated)?\n- Results come from the recency-skewed LIRS trace only; do they generalize to other traces?\n\n## Per-policy notes\n\n- DASH segment promotion policy (the only policy probed so far): base for future work is the bit-packed second-hit filter at baseline item size. Prefer richer free-metadata admission signals over more insert-position variants.\n"
}
```

### 19:47:11 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four fixed sections in order. It is built only from the two ledger cycles and quotes no numbers.

- **Recurring wins:**
  - Admission changes pay off more reliably than insert-position changes.
  - The bit-packed second-hit filter is the best base so far. It kept the hit-rate gain, restored baseline item size and improved latency.
  - Keeping one-hit-wonders out of the hot tiers is the theme behind every gain.
- **Recurring failures:**
  - Stacking position levers (cold first admission, cold demotion, flag reset) on admission or uniform MRU gave weak or mixed results, because the levers overlap.
  - Position-only tweaks look close to exhausted.
  - Several latency claims rest on inference rather than measurement.
- **Open questions:**
  - Can the spare bits hold a small hit counter or an age bit?
  - Does the filter keep its latency gain with a milder demotion penalty?
  - Are the small hit-rate gains real or trace noise?
  - Does uniform MRU's gain come from promotion or from demotion cascades?
  - Do the results generalize beyond the recency-skewed LIRS trace?
- **Per-policy notes:** DASH segment promotion is the only policy probed so far. The note points the next cycle at the bit-packed filter as its base, and at free-metadata admission signals over more insert-position variants.


---

- finished: 2026-10-20T19:47:12+00:00
- duration: 16.20s
- status: ok
- input_tokens: 4
- output_tokens: 1233
- cache_read_tokens: 35082
- cache_write_tokens: 6376
- cost_usd: 0.041628
