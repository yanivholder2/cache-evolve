# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0001/cross-cycle
- allow_edits: True
- started: 2026-10-28T11:46:00+00:00

## timeline

### 11:46:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0001/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n_Projection of 1 completed cycle (0001). Patterns below are provisional until later cycles confirm them._\n\n## Recurring wins\n\n- Soft scan resistance pays off. A one-time block is held in a probation area or given an unset reference bit instead of being rejected outright. SLRU (probation + protected) was the only cohort Winner. It improved hit-rate on every trace and size.\n- Segmentation gains grow with capacity. More room lets the protected segment hold the frequently reused working set.\n- Frequency-aware filtering helps consistently on the frequency-skewed trace (hm_0) at all sizes.\n\n## Recurring failures\n\n- Hard admission gates hurt recency-driven workloads. Plain TinyLFU in front of LRU, with a strict-greater reject on count ties, lost on web_0, worst at small capacity. Any mechanism that delays or blocks a block's second access loses short-range reuse.\n- A plain CLOCK ring at large capacity decays toward FIFO because its reference bits stay set. This produced a slight web_0 loss.\n- Every fitness vector so far has held hit-rate only. `metadata-size` and `per-op-time` were never measured, so no cost-axis claim has been verified.\n\n## Open questions\n\n- Does SLRU keep its edge once `metadata-size` and `per-op-time` are measured? Does CLOCK's expected cost advantage make up for its small hit-rate deficit?\n- Would a larger or adaptive probation share at small capacity recover blocks whose reuse distance is longer than probation?\n- Can admission filtering help recency-heavy traces (web_0)? Candidate designs: W-TinyLFU, which puts an admission-free recency window in front of SLRU, or a sketch that admits low-count ties.\n- Can CLOCK-based segmentation keep CLOCK's cheap `get` and still match SLRU's hit-rate?\n\n## Per-policy notes\n\n- **SLRU:** Current best lineage. The next step is to tune the probation/protected split, especially at small size, or try an adaptive split.\n- **CLOCK:** Kept as an option for the cost axes; its hit-rate is roughly LRU-level. Its large-ring degeneration on web_0 needs a fix, such as segmentation or a bit-decay variant.\n- **TinyLFU (over plain LRU):** Do not retry the v003 form. Use the sketch only behind a recency window, with lenient tie admission.\n- **Traces:** hm_0 rewards frequency filtering. web_0 rewards short-range recency, most strongly at small capacity. A good policy has to serve both.\n"
}
```

### 11:46:16 chat

I wrote `cross-cycle-insights.md` with the four required sections. The ledger has only one completed cycle (0001), so nothing has held up across cycles yet. I labelled every point as provisional, to be confirmed by later cycles.

- **Recurring wins:** Gentle filtering of blocks seen once paid off: SLRU (v001), which holds them on a probation list, was the only Winner, and its gain grew with cache size. Filtering by how often a block is reused helped on hm_0 at both sizes.
- **Recurring failures:**
  - A strict admission gate (TinyLFU in front of LRU, v003) hurt web_0, which depends on blocks being reused soon after first access.
  - A plain CLOCK ring (v002) at large size drifts toward FIFO, slightly losing on web_0.
  - No variant measured `metadata-size` or `per-op-time`, so every cost claim is still unverified.
- **Open questions:** How SLRU and CLOCK compare once costs are measured; whether a larger or adaptive probation share helps at small size; whether W-TinyLFU (a small unfiltered window in front of SLRU) or admitting low-count ties can help web_0; and whether a segmented CLOCK keeps CLOCK's cheap `get`.
- **Per-policy notes:** SLRU is the line to build on, CLOCK is kept as an option for the cost measures, and the v003 TinyLFU design should not be retried. There are also short notes on what each trace rewards.

The file is in `cycles/0001/cross-cycle/`.


---

- finished: 2026-10-28T11:46:16+00:00
- duration: 16.22s
- status: ok
- input_tokens: 4
- output_tokens: 1530
- cache_read_tokens: 34520
- cache_write_tokens: 6858
- cost_usd: 0.092384
