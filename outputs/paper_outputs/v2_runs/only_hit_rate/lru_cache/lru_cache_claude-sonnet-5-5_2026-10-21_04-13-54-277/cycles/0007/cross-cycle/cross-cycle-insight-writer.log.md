# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:41:22+00:00

## timeline

### 04:41:32 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Segmented LRU (scan-resistant) was the first real gain and remains the base for the best lines; every later winner builds on it.\n- Raising the protected share helped over two cycles (larger beat smaller), then flattened. The curve is now at a plateau.\n- Admission filtering (sketch gate) gave small gains; counting rejected misses matters because recurring keys build frequency across retries.\n- Simpler designs stay competitive: a doorkeeper-free SLRU plus sketch won against more elaborate variants.\n- CLOCK-style rewrites give small hit-rate gains over strict LRU, but they are incremental, not transformative.\n\n## Recurring failures\n\n- Weakening the protected segment (smaller share, direct ghost-to-protected admission) lost hit-rate.\n- Probation-side tweaks (demotion placement, probation-only eviction scans, ghost at probation MRU) were inert or matched the parent. Demotions are rare at the high protected share, and some variants were dead by construction.\n- Protected-eviction refinements (second-chance bit) and approximate-recency shortcuts (relink skip) gave nothing or lost a little.\n- Admission-gate tuning (not counting rejected misses, doorkeeper, longer aging window) is saturated. Slower decay lets stale high-count keys block newer candidates.\n- Strict-LRU micro-optimizations are neutral on hit-rate and cannot win while cost is unmeasured.\n- Single-constant tweaks are validated on one trace at one cache size, so differences are marginal.\n\n## Open questions\n\n- Metadata-size and per-op-time have been missing from every fitness vector for seven cycles. Is this a harness fault? It blocks judging every cost-motivated variant (compact layout, fast paths, CLOCK, sketch, doorkeeper).\n- Do the gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure? Only one scan-heavy trace has been used.\n- Would adaptive segment sizing (ARC-style) beat any fixed share?\n- Can frequency that persists across segments, or a CLOCK/sketch hybrid, beat the segmented line in a head-to-head, cost-aware comparison?\n- Do the aging window and doorkeeper reset have separate effects? They were changed together.\n- Do the compact layout and cheap promotion ideas combine with the segmented winner?\n\n## Per-policy notes\n\n- Strict LRU: the baseline. Hit-rate-preserving rewrites tie it and cannot win.\n- Segmented LRU: the best line. The protected share is saturated near 90-95%. Probation is near its minimum useful size and the segment is saturated on this trace.\n- Segmented LRU plus ghost list: the direct-to-protected version hurt. The gentler version matched its parent. Drop it unless long-distance reuse is added.\n- Sketch admission (TinyLFU-style): marginal gain. Counting rejected misses helps. The doorkeeper is roughly cost-neutral. Keep the aging window near its current length. The doorkeeper-free SLRU plus sketch is the simplicity baseline.\n- Frequency-aware victim selection: now has a real signal, but only a small gain. The probation LRU head is already near the best victim.\n- CLOCK (one-bit, frequency-aware, plus sketch gate): small hit-rate gains. Size and speed advantages are unverified.\n"
}
```

### 04:41:34 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four fixed sections in order, built only from the cycle 1–7 ledger digest.

The main points it records:
- **Recurring wins:** segmented LRU is the base for the best lines. The protected-share gain is now at a plateau, and a simpler design (SLRU plus a sketch gate) stays competitive.
- **Recurring failures:** weakening the protected segment lost hit-rate. Probation-side tweaks, protected-eviction refinements and admission-gate tuning were inert or saturated.
- **Open questions:**
  - Metadata-size and per-op-time have been missing from every fitness vector for seven cycles, so no cost claim has been checked.
  - Only one scan-heavy trace at one cache size has been used.
  - Adaptive segment sizing and frequency kept across segments are untried.
- **Per-policy notes:** these are grouped by strict LRU, segmented LRU, ghost list, sketch admission, frequency-aware victim selection and CLOCK.


---

- finished: 2026-10-21T04:41:35+00:00
- duration: 13.05s
- status: ok
- input_tokens: 4
- output_tokens: 1629
- cache_read_tokens: 38966
- cache_write_tokens: 10791
- cost_usd: 0.064026
