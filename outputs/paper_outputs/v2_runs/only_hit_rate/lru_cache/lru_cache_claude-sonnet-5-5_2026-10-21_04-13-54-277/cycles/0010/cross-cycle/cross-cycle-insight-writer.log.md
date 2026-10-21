# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:56:02+00:00

## timeline

### 04:56:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Moving from strict LRU to a segmented LRU (SLRU) with scan resistance gave the one clear gain (cycle 1). Frequency-gated promotion and admission (a sketch gate) is the other structural benefit. Rebuilds from baseline keep rediscovering both.\n- Raising the protected share helped until about 90–95%, then flattened (cycles 2–4).\n- Counting rejected misses in the sketch is a safe, helpful rule. Removing it lost hit-rate. A stricter admission margin is safe only because of this rule.\n- Simpler designs stay competitive: doorkeeper-free SLRU plus a sketch is a solid baseline.\n- Every later win was marginal, so the incumbent looks near a local optimum on this workload.\n\n## Recurring failures\n\n- Weakening the protected segment lost hit-rate: a smaller share, or ghost-hit direct-to-protected admission.\n- Probation-side tweaks were inert or dead by construction: demotion placement, protected-eviction second-chance bits, probation-only frequency scans, and ghost lists. Demotions and fallback paths rarely run at a high protected share.\n- Sketch and gate parameters are bracketed on both sides and plateaued: aging window (longer and shorter both lost), count scale versus margin (only their ratio matters), doorkeeper, rejected-miss counting, and victim scan or rotation.\n- Adaptive segment sizing and deferred admission windows found no variance to exploit.\n- Cheaper-per-op shortcuts, such as skipping relinks, cannot be judged, and they cost a little hit-rate.\n- Single-constant tweaks without a second trace to validate them give noise-level selection.\n\n## Open questions\n\n- Metadata-size and per-op-time have been missing from the fitness vectors in all ten cycles. Is it a harness fault? Until it is fixed, claimed savings (CLOCK, the 4-bit sketch, a doorkeeper-free design, relink-skip) are unverified, and selection rests on hit-rate alone.\n- Do any gains hold on other traces or cache sizes? All evidence comes from one scan-heavy trace at one size. Long-distance reuse and low-scan traces are untested, and they may revive ghost lists, adaptive sizing and admission windows.\n- Can a different ranking signal (size or cost awareness, or a different policy family) move the plateau?\n- What is the head-to-head, cost-aware result of CLOCK + sketch against the segmented incumbent?\n- Does frequency that persists across segments, or covers protected entries, add anything?\n\n## Per-policy notes\n\n- Strict LRU: only a baseline. Micro-optimizations that preserve its semantics cannot win on hit-rate.\n- Segmented LRU (about 95% protected): the incumbent line, saturated on this trace. Remaining tweaks (margin, rotation, 4-bit conservative sketch) are optional and marginal.\n- Sketch admission gate (TinyLFU-style): small gain. The extra hashing, counters and decay cost is unmeasured.\n- CLOCK variants (1-bit, 2-bit frequency, CLOCK + sketch): repeatedly show an edge over strict LRU but never beat the segmented line. They are incremental on hit-rate, and their claimed value (cheap hits, small nodes) is unmeasured.\n- ARC-style adaptive sizing and ghost lists: noise-level or negative, with extra bookkeeping.\n"
}
```

### 04:56:19 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory, with the four fixed sections in the required order. It is built only from the ten ledger digests and states no numbers.

- **Recurring wins:** moving from strict LRU to a segmented LRU, and frequency-gated promotion and admission, produced the clear gains. A higher protected share helped until about 90–95%, then flattened. Every later win was marginal.
- **Recurring failures:** weakening the protected segment lost hit-rate. Probation-side tweaks, ghost lists, adaptive sizing and deferred admission windows did nothing. Sketch and gate parameters are plateaued from both sides.
- **Open questions:**
  - Metadata-size and per-op-time have been missing in all ten cycles, so selection rests on hit-rate alone.
  - Every result comes from one scan-heavy trace at one cache size.
  - Whether a different ranking signal could move the plateau.
  - The head-to-head, cost-aware comparison of CLOCK + sketch against the segmented incumbent.
- **Per-policy notes:** these cover strict LRU, the segmented LRU incumbent, the sketch admission gate, the CLOCK variants and the ARC-style adaptive sizing.


---

- finished: 2026-10-21T04:56:19+00:00
- duration: 17.41s
- status: ok
- input_tokens: 4
- output_tokens: 1727
- cache_read_tokens: 41292
- cache_write_tokens: 13137
- cost_usd: 0.074855
