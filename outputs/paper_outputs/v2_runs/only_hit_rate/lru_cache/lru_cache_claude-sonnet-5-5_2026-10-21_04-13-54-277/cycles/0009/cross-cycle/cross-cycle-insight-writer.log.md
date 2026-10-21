# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:51:40+00:00

## timeline

### 04:51:51 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Segmented LRU (probation + protected) is the one structural change that clearly paid off. Early cycles gained from it, and later gains came from tuning its protected share upward until the curve flattened.\n- Simple designs stay competitive. The doorkeeper-free SLRU + sketch gate matched or beat more elaborate variants.\n- Counting rejected misses in the sketch helps: recurring keys build frequency across retries and earn admission. Removing it hurt.\n- Frequency that persists across segments gives frequency-aware decisions a real signal, but the gain is small.\n- Winners in recent cycles are mostly marginal hit-rate edges, so the selection signal is thin.\n\n## Recurring failures\n\n- Cheaper-per-op and smaller-metadata ideas (compact layout, fast paths, relink-skip, 4-bit sketch) cannot be judged. Metadata-size and per-op-time have been missing from every fitness vector in every cycle.\n- Tuning single constants past their optimum is inert or slightly negative. This covers protected share, aging window (bracketed from both sides), admission margin, and doorkeeper reset.\n- Probation-side tweaks (demotion placement, probation-only eviction scans, victim scans) are nearly inert because demotions are rare at a high protected share. One variant was dead by construction because frequency was never persisted.\n- Ghost lists and adaptive sizing either hurt (direct-to-protected) or match the parent. The trace shows little phase variance to exploit.\n- Deferred admission (small window) and extra admission machinery add complexity without return, since entry-time gating is nearly sufficient.\n- Every cycle's results rest on one scan-heavy trace at one cache size, so many \"plateau\" conclusions may be trace-specific.\n\n## Open questions\n\n- Is the missing metadata-size and per-op-time measurement a harness fault? It has blocked cost judgment for nine cycles and should be fixed before more hypotheses.\n- Do any gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure?\n- Would a different axis help, such as an eviction signal based on size or cost, or frequency that spans protected entries?\n- Does CLOCK's cheaper hit path offset its miss-time sweep and sketch updates in a head-to-head cost-aware comparison with the segmented incumbent?\n- Is the compact sketch or the admission-margin tweak worth keeping once cost is measured, or are they noise?\n\n## Per-policy notes\n\n- Strict LRU: Micro-optimizations that preserve semantics cannot win on hit-rate and were unmeasurable on cost.\n- Segmented LRU: Protected share rose to a plateau around the high-90s percent range. It is saturated on this trace. Smaller protected shares and unconditional ghost-to-protected admission lost.\n- Segmented LRU + sketch gate: The doorkeeper-free version is the simplicity baseline. Aging window, doorkeeper, margin and counter precision are all within noise.\n- CLOCK variants: These repeatedly report a hit-rate edge over strict LRU but have not beaten the segmented line, and frequency bits add only incremental gains. Their claimed size and speed advantages remain unverified.\n- ARC-style adaptive sizing: Noise-level on this workload.\n"
}
```

### 04:51:54 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four required sections in the fixed order. It's built only from the nine ledger digests. I didn't read the previous insights file or any code.

The main points it records:
- **Cost metrics:** Metadata-size and per-op-time have been missing from every fitness vector for nine cycles. Any claimed size or speed gain is unverified, and the file asks for this to be fixed before more hypotheses.
- **What worked:** The segmented LRU, with its protected share tuned upward until the gains flattened, is the one structural change that clearly paid off. The simple SLRU plus sketch gate, with no doorkeeper, stayed competitive.
- **What failed:** Single-constant tuning past its optimum, probation-side tweaks, ghost lists, adaptive sizing and deferred admission were all inert or slightly negative.
- **Evidence limits:** Every result comes from one scan-heavy trace at one cache size, so the "plateau" conclusions may be specific to that trace.


---

- finished: 2026-10-21T04:51:54+00:00
- duration: 14.37s
- status: ok
- input_tokens: 4
- output_tokens: 1584
- cache_read_tokens: 40563
- cache_write_tokens: 12353
- cost_usd: 0.070144
