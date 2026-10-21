# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:44:35+00:00

## timeline

### 04:44:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Segmented LRU (probation + protected) was the one early change that produced a real hit-rate gain. Later cycles kept building on it.\n- Raising the protected share helped through about 90%, then flattened near 95%. Fixed-share tuning is exhausted.\n- Scan resistance is what pays on this workload. It came from segmentation, from CLOCK's unreferenced insertion, and from a sketch admission gate.\n- Simpler designs stay competitive. The doorkeeper-free SLRU + sketch rebuild won, and the compact conservative-update sketch won by a hair.\n- Counting rejected misses in the sketch helps. Recurring keys build frequency across retries.\n\n## Recurring failures\n\n- Weakening the protected segment hurt: a smaller share, and ghost hits admitted straight to protected.\n- Probation-side tweaks were inert or tied with the parent: demotion placement, probation-only frequency scans, second-chance on protected eviction. Demotions are rare at a high protected share, and some variants were dead by construction because frequency was never persisted.\n- Admission-gate refinements gave marginal or negative results: the doorkeeper, not counting rejected misses, a longer aging window, and a deferred-admission window. Stale high counts and under-counted new hot keys cancel the benefit.\n- Adaptive (ARC-style) sizing found no phase variance to exploit.\n- Approximate-recency shortcuts (skipping relinks) cost a little hit-rate. Their speed benefit was never measured.\n- Single-constant tweaks produce only noise-level gains, and \"winner\" status often rests on a marginal edge.\n\n## Open questions\n\n- Metadata-size and per-op-time have been missing from every fitness vector for eight cycles. Is this a harness fault? Until it is fixed, claimed cost wins (compact nodes, 4-bit sketch, CLOCK's cheap hits, fast paths) are unverified and selection uses hit-rate alone.\n- Do any gains hold on other traces or cache sizes? Everything so far comes from one scan-heavy trace at one size. Long-distance reuse and low scan pressure are untested.\n- Is the current incumbent (SLRU + compact sketch gate) at a true optimum or only a local one? A different eviction signal might beat it, such as size or cost awareness, or frequency persisting across segments.\n- How does CLOCK + sketch compare head to head with the segmented winner, once costs can be measured?\n- Is the sketch gate worth its time and memory over plain SLRU?\n\n## Per-policy notes\n\n- Strict LRU: micro-optimizations are neutral on hit-rate and cannot win without cost metrics.\n- Segmented LRU: the best-performing line. It is plateaued on protected share, probation placement, eviction scans and ghost lists. Further tuning is low value.\n- CLOCK / frequency-CLOCK: a small hit-rate gain over strict LRU, incremental and not transformative. Its main claimed advantage, cheaper metadata and hits, is unmeasured.\n- Sketch admission (TinyLFU-style): a marginal gain on top of SLRU. The simple version with rejected misses counted is enough. The doorkeeper, longer aging and window variants add cost for little return. A 4-bit conservative-update sketch is the current compact incumbent, pending cost verification.\n- Adaptive / ARC-style: not useful on this trace.\n"
}
```

### 04:44:49 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four sections in the required order. It's built only from the eight-cycle ledger digest.

- **Recurring wins:** Segmented LRU is the one early change that paid off. Higher protected shares helped up to roughly 90% and then flattened. Scan resistance is what pays on this workload. Simpler designs stay competitive.
- **Recurring failures:** Weakening the protected segment hurt. Probation-side tweaks and admission-gate refinements were inert or marginal. Adaptive sizing and approximate-recency shortcuts gave nothing measurable.
- **Open questions:** The biggest is that metadata-size and per-op-time have been missing from every fitness vector for eight cycles, so claimed cost wins are unverified. All results also come from one scan-heavy trace at one cache size.
- **Per-policy notes:** These are grouped by strict LRU, segmented LRU, CLOCK, sketch admission and adaptive sizing.

The file keeps to qualitative terms and contains no invented numbers.


---

- finished: 2026-10-21T04:44:49+00:00
- duration: 14.59s
- status: ok
- input_tokens: 4
- output_tokens: 1635
- cache_read_tokens: 39761
- cache_write_tokens: 11584
- cost_usd: 0.067416
