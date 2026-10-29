# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:06:57+00:00

## timeline

### 12:07:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Each cycle's Winner built on the previous one: SLRU (c1), then W-TinyLFU over SLRU (c2), then a tightened sketch (c3), then a capped hill-climbed window (c4). Building on the current champion has worked every time.\n- Soft, relative signals beat hard overrides (c1–c4). Examples are SLRU probation, a duel against the current victim with ties admitted, and a gradually adapting window.\n- Strict one-shot filtering is what drives hm_0, especially at small capacity. The pieces are the probation segment, the sketch admission gate, dedup on miss-then-put, the doorkeeper, the long reset period and a wide sketch. Together these are the most dependable hm_0 gains (c1–c4).\n- Enlarging the front segment (window or HIR queue) is the only lever that consistently lifts small-size web_0 (c3, c4). Adapting the window size with absolute caps on its knobs, plus a floor, fixed this cell without losing anything elsewhere (c4).\n- Changing one lever per variant gives clean attribution (c4). Bundled changes in earlier cycles hid the causes.\n\n## Recurring failures\n\n- Hard admission or promotion rules lose on web_0's short-range recency reuse. Examples: TinyLFU over LRU with a strict-greater reject (c1), ghost hit → protected (c2), and an unconditional bypass on in-window reuse (c4).\n- Parameters fixed as a share of capacity or scaled linearly with it fail at one end of the size range. Examples: the 1% window and HIR share (c2), the capacity-sized ghost list (c2), and the climber's step and period tied to capacity (c3).\n- Loosening frequency memory, through shorter resets or faster decay, gives back hm_0 gains at small size and barely helps web_0 (c3, c4).\n- LIRS tuning only trades hm_0 against web_0 and never beat the W-TinyLFU line (c2, c3).\n- In all four cycles only hit-rate was measured. `metadata-size` and `per-op-time` were never reported, so every cost claim is unverified.\n\n## Open questions\n\n- Which climber cap does the work, the sample-period cap or the step cap? How far does the window actually move in each trace/size cell?\n- With an adaptive window in place, does the doorkeeper still cost anything on web_0?\n- Can in-window reuse help as a bounded duel bonus or tie-breaker instead of a bypass?\n- How do the sketch, the doorkeeper, the climber's per-access sampling and the ghost/LIRS history compare to plain SLRU on the cost axes? Does CLOCK's expected cost advantage hold?\n- Is there one front-segment share that works for both traces, or must it always adapt?\n\n## Per-policy notes\n\n- **LRU (baseline):** every variant since c1 has beaten it.\n- **SLRU:** a robust base whose gains grow with capacity (c1). A ghost list feeding protected hurt (c2). If ghost history comes back, it should only feed the admission comparison.\n- **CLOCK:** slightly worse on large-size web_0 because its bits stay set and it decays toward FIFO (c1). Kept only for its unmeasured cost advantage.\n- **TinyLFU / W-TinyLFU:** plain TinyLFU over LRU failed (c1). With a window in front of SLRU and ties admitted, it has been the lineage of every Winner since c2. Keep the current sketch settings (long reset, doorkeeper, dedup, wide width) and the capped climber with its floor.\n- **LIRS:** competitive with LRU but never beat W-TinyLFU. Changing the HIR share is a trade between traces, not a gain. Dropped as of c3.\n"
}
```

### 12:07:18 chat

I've written `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder. It uses the four required sections in order, and is built only from the ledger for cycles 1–4.

- **Recurring wins:** Each cycle's Winner built on the previous one: SLRU, then W-TinyLFU, then a tighter sketch, then a capped adaptive window. Soft, relative admission signals kept beating hard rules. Strict filtering of blocks seen only once drives hm_0. Enlarging the front segment is the only lever that reliably lifts small-size web_0.
- **Recurring failures:** Hard admission or promotion rules lose on web_0. Settings tied to a fixed share of capacity fail at one end of the size range. Shorter sketch resets give back hm_0 gains. LIRS tuning only trades one trace against the other.
- **Unmeasured costs:** In all four cycles, only hit-rate was measured, so every claim about `metadata-size` or `per-op-time` is still unchecked. The file lists this as a recurring failure.
- **Open questions:** Which of the climber's two caps does the work, and how far the window actually moves in each trace and size. Whether the doorkeeper still hurts web_0 now that the window adapts. Whether in-window reuse helps as a small bonus in the admission comparison rather than a bypass. How the designs compare once the cost axes are measured.
- **Per-policy notes:** Short notes on LRU, SLRU, CLOCK, TinyLFU/W-TinyLFU and LIRS.


---

- finished: 2026-10-28T12:07:18+00:00
- duration: 21.48s
- status: ok
- input_tokens: 4
- output_tokens: 1920
- cache_read_tokens: 38218
- cache_write_tokens: 9689
- cost_usd: 0.123572
