# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:31:30+00:00

## timeline

### 14:31:48 thinking



### 14:31:53 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_14-00-21-467/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Eviction-neutral changes (lookup collapse, sentinel index, relink skip, index pre-sizing, hasher swap) have repeatedly left hit-rate exactly equal to their parent. This is a dependable non-regression, not a measured gain. Use these as safe bases for stacking.\n- Bounded frequency-aware eviction (least-hit entry within a small tail window) is the only change that has moved hit-rate upward, and only slightly (cycle 0007). It is a single-cycle signal, not yet a recurring win.\n- No measured gain has held across more than one cycle.\n\n## Recurring failures\n\n- Winner labels are often not backed by the measured dimension. Across cycles 0001–0006 several Winners had no reported gain on their target, and in cycles 0005 and 0006 the named Winner had a hit-rate drop. The selection criterion is not tracking the fitness signal.\n- Per-op time and metadata size have never been recorded in any cycle (0001–0007). Every hypothesis motivated by time or size is therefore untested.\n- Loosening strict recency lowered hit-rate on workload-a at size 2048: the segmented probation/protected list and second-chance referenced-bit eviction (cycle 0005) both dipped, and second-hit promotion (cycle 0006) also lost.\n- Tail insertion of new keys lost hit-rate in both of its cycle 0006 forms, despite a gain in cycle 0005. It cannot be counted as reliable.\n- Ghost-set admission filters have held hit-rate or produced no usable result (cycles 0002, 0003, 0007). No cycle has shown the filter engaged, so the idea is untested rather than refuted.\n- Infrastructure rejections (two variants in cycle 0001, one in cycle 0004) are not strategy evidence. Those approaches remain unexamined.\n\n## Open questions\n\n- Does tail insertion help on its own against a fixed parent? Cycles 0005 (gain), 0006 (loss), and 0007 (null) cannot be reconciled from the insights.\n- What should the Winner criterion require? Several Winners lacked a movement on their own target dimension.\n- Does the frequency-aware tail-window gain hold on a second trace or cache size, and what does its per-entry counter cost in memory?\n- Did the ghost filter and the tail-insertion overlay ever engage? Engagement counters are needed before either can be ruled in or out.\n- The H03 label has meant different mechanisms across cycles (per-op check, pre-sized index, hasher swap, segmentation). Fix one definition before lineages cite it.\n- Which direction did the cycle 0004 segmentation variant move hit-rate, and by how much? The insight does not say.\n- Does the protected cap or demotion rule in segmentation carry headroom, or is segmentation dead as written?\n\n## Per-policy notes\n\n- Strict LRU (move-to-front recency): representation and lookup changes preserve it. Every change that loosened recency order (segmentation, second-chance, second-hit promotion) lowered hit-rate on workload-a at size 2048, which suggests strict recency is doing useful work on this trace.\n- LRU with insertion-position changes (tail or LRU-end insertion, sparse MRU insert): mixed evidence across cycles 0005–0007 (gain, losses, null). Unresolved and not safe to build on until tested alone.\n- Segmented probation/protected: dropped as written after a hit-rate dip. Its cap and demotion parameters have not been tested in isolation.\n- Second-chance referenced-bit: lowered hit-rate in cycle 0005. No current evidence for it.\n- Ghost-set admission: never shown to engage. Where reported, it was neutral or had no usable result. Untested.\n- Frequency-aware bounded tail-window eviction: the only upward hit-rate movement, and slight. Memory and per-op cost are unmeasured. Lead candidate, pending a second trace or size.\n"
}
```

### 14:31:56 chat

I wrote `cross-cycle-insights.md` to the cycle 0007 cross-cycle output directory, using the four required sections. I built it only from the inlined ledger digest and did not read the previous file.

The main points:

- **Recurring wins:** The only pattern that holds across cycles is that eviction-neutral layout and lookup changes keep hit-rate equal to their parent. That is a safe non-regression, not a measured gain. The one upward hit-rate move, from bounded frequency-aware eviction in cycle 0007, is slight and comes from a single cycle.
- **Recurring failures:** Several named Winners had no reported movement on their target dimension, and two had a hit-rate drop. Per-op time and metadata size have never been recorded in any cycle. Loosening strict recency lowered hit-rate. Tail insertion's evidence conflicts across cycles 0005–0007.
- **Open questions:** These cover tail-insertion reconciliation, the Winner criterion, whether the frequency-aware gain generalises, whether the ghost filter and tail overlay ever engaged, and the H03 definition, which has drifted across cycles.
- **Per-policy notes:** Grouped by strict LRU, insertion-position changes, segmentation, second-chance, ghost-set admission, and frequency-aware tail-window eviction.


---

- finished: 2026-10-20T14:31:56+00:00
- duration: 26.20s
- status: ok
- input_tokens: 4
- output_tokens: 5329
- cache_read_tokens: 38776
- cache_write_tokens: 14210
- cost_usd: 0.005895
