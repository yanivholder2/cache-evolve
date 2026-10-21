# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:33:08+00:00

## timeline

### 04:33:17 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan resistance is the lever that pays. Segmenting the LRU, then CLOCK with an unreferenced-insert bit, then an admission gate each produced a small hit-rate gain.\n- Growing the protected share helped monotonically at first (smaller lost, larger won), then flattened out past about 90–95%.\n- Each cycle's winner was a simple, single-mechanism change. Gains are always small.\n\n## Recurring failures\n\n- Strict-LRU micro-optimizations (compact layout, fast paths) can't win. They match the parent's hit-rate and their cost claims are never measured.\n- Tweaks to the protected side (smaller share, second-chance on protected eviction, relink-skip) either lost hit-rate or matched the parent. The protected side is saturated.\n- Probation-side tweaks (placement of demoted nodes, frequency-aware scans, probation-MRU ghost) are inert. Demotions are rare at a high protected share. One frequency variant was dead by construction because its counts never persisted.\n- Unconditional ghost-to-protected admission hurt hit-rate. Gentler ghost variants only recovered to parity.\n- Ideas judged on predicted size or speed can't be selected. Metadata-size and per-op-time have been missing from every fitness vector for five cycles.\n\n## Open questions\n\n- Is the missing metadata-size and per-op-time measurement a harness fault? It blocks every cost-motivated idea (CLOCK's advantage, sketch cost, relink-skip) and is the top blocker.\n- Do the gains hold on other traces and cache sizes? All evidence comes from one scan-heavy trace at one size. Long-distance-reuse and low-scan traces are untested.\n- Would adaptive segment sizing (ARC-style) beat any fixed share?\n- Can CLOCK or frequency ideas combine with the segmented design? Would frequency that persists across segments help?\n- Is the admission gate worth its cost? Cheaper forms include not counting rejected misses, or a tiny doorkeeper.\n\n## Per-policy notes\n\n- Segmented LRU: protected share is tuned out at about 95%. Protected-eviction and probation-placement refinements are exhausted on this trace.\n- CLOCK (1-bit): small hit-rate gain from scan resistance without list manipulation. Its size and speed benefits are unverified.\n- Frequency CLOCK (2-bit): only an incremental gain over 1-bit CLOCK.\n- Admission gate (sketch): marginal gain on top of segmentation. Its hashing, counter and decay costs are unmeasured, and it can reject new hot keys.\n- Ghost lists: harmful when they promote directly, neutral when gentle. They are worth revisiting only with long-reuse traces.\n- Strict LRU: no further micro-optimizations until cost metrics are measured.\n"
}
```

### 04:33:20 chat

I wrote `cross-cycle-insights.md` to the cycle 0005 cross-cycle output directory. It has the four required sections in order, built only from the five ledger digests.

- **Recurring wins:** Scan resistance is what has paid off: segmenting the LRU, the CLOCK variants and the admission gate each gave small hit-rate gains. A larger protected share helped until it flattened out around 90–95%.
- **Recurring failures:**
  - Strict-LRU micro-optimizations and protected-side tweaks didn't win.
  - Probation-side changes had no effect.
  - Ghost lists either hurt hit-rate or matched the parent.
  - Metadata-size and per-op-time have been missing from every fitness vector for five cycles, so cost-motivated ideas can't be selected.
- **Open questions:** The top one is whether the missing metadata-size and per-op-time measurement is a harness fault. The rest are whether gains hold beyond the one scan-heavy trace at one cache size, adaptive segment sizing, combining CLOCK or frequency ideas with segmentation, and whether the admission gate is worth its cost.
- **Per-policy notes:** These cover segmented LRU, 1-bit CLOCK, 2-bit frequency CLOCK, the sketch admission gate, ghost lists and strict LRU.

The file uses qualitative terms only, with no invented numbers.


---

- finished: 2026-10-21T04:33:21+00:00
- duration: 13.39s
- status: ok
- input_tokens: 4
- output_tokens: 1556
- cache_read_tokens: 37375
- cache_write_tokens: 9009
- cost_usd: 0.05585
