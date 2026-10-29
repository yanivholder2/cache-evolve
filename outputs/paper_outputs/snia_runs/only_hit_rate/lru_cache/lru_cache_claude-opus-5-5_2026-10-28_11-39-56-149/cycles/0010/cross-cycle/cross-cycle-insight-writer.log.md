# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:51:19+00:00

## timeline

### 12:51:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **Lineage:** LRU → SLRU (c1) → W-TinyLFU over SLRU with tie-admitting duel (c2) → tightened sketch: dedup, wide, doorkeeper, long reset (c3) → capped hill-climbed window with floor (c4) → large-only initial step (c7) → large-only 75% protected split (c8) → small-only 32× `reset_at` (c9) → conservative update (c10, fragile). Every real gain was an extension of this line.\n- **Soft, relative signals beat hard overrides** (c1, c2, c4, c5): the duel against the current victim with ties admitted works. Hard gates, bypasses and full-status promotion all lost.\n- **Strict one-shot filtering is what wins hm_0** (c1, c3, c4, c7): dedup, doorkeeper and the long reset all carry real weight. Small-size hm_0 is the most fragile cell.\n- **Enlarging the front window is the main lever for recency-heavy web_0** (c3, c4). The capped adaptive climber gave the run's only all-cell gain.\n- **Changing one lever per variant makes results attributable** (c4 onward). Bundled changes (c3) could not be diagnosed.\n- **Size gates are reliable** (c7, c8, c9): cells on the other side of the gate stay exactly unchanged, so you can tune per size without collateral effects.\n- **Capping parameters at absolute values works better than scaling them with capacity** (c2, c3, c4). Settings tied to capacity keep failing at one end of the size range.\n- **Longer sketch memory helped at small size** (c9): the first lever that lifted both small traces together. Its benefit has now peaked (c10).\n\n## Recurring failures\n\n- **Loosening admission** (c1 hard gate aside, c4 bypass, c5 window-reuse bonus, c7 random admission): this lost every time, and hm_0 paid first. The tie-admitting duel is already lenient enough.\n- **Counting the same evidence twice:** in-window reuse is already in the sketch, so bonuses or bypasses based on it skew close duels toward recency (c4, c5).\n- **Ghost or non-resident history used as promotion** (c2): one return from outside the cache is weak evidence on web_0.\n- **Tuning the climber** (c5, c6): the ceiling, shorter or longer periods, a bigger step cap, hysteresis and flip damping (c8) were all neutral or worse. The climber is at a local optimum.\n- **Draining protected faster** (c7 global split, c9 frequency-based victim, c9 split climber): protected residency carries weight for both traces at large size.\n- **Large-size static gates trade web_0 against hm_0** (c7, c8, c10). Step, split, damping and longer memory all move along that same trade-off. No lever has helped both large traces together.\n- **Faster sketch aging** (c4) and **longer aging past its peak** (c10, both sizes) both hurt.\n- **Resizing the doorkeeper** (c8): a more accurate doorkeeper removes the uniform first-touch bonus and hurts hm_0.\n- **LIRS** (c2, c3): tuning its HIR share only moves hit-rate between traces. Dropped.\n- **Measurement debt:** `metadata-size`, `per-op-time` and `win_cap` have not been reported in any of the 10 cycles. Every cost claim is unverified.\n\n## Open questions\n\n- Does strict-beat for warm victims (count > 1) keep c10's web_0 gain and recover hm_0? Tie strictness is the hidden hinge behind several results (c1, c2, c10).\n- Should conservative update be gated to small size only? Its large-size cells got nothing or lost.\n- Doorkeeper fill and `reset_at` are linked (c8, c10). Would clearing the doorkeeper on its own shorter schedule let long memory stop leaking +1 bonuses to one-shot keys?\n- At large size, is a shorter `reset_at` (around 8×) the right direction?\n- Can a clean workload signal (a ghost hit or a shadow comparison) drive protected-split adaptation at large size, where raw hit-rate climbers failed?\n- Where does `win_cap` settle in each cell, and what do the sketch, window and climber cost per op and in metadata?\n\n## Per-policy notes\n\n- **LRU (baseline):** beaten by every later design. Use it only as the reference point.\n- **SLRU:** a soft probation filter that helps more as capacity grows. The large protected segment is the critical part, and 80% small / 75% large is the current split. Global split changes and frequency-based victim rules hurt.\n- **CLOCK:** decayed toward FIFO on web_0 at large size. Its possible cost advantage was never measured. Dormant.\n- **TinyLFU (plain over LRU):** a strict-greater reject hurts recency traces. Do not retry.\n- **W-TinyLFU over SLRU (current line):** keep the tie-admitting duel, dedup, wide sketch, doorkeeper, long reset (32× small, 16× large), capped climber with floor, and large-only step and split gates. Conservative update is the newest addition and is fragile.\n- **LIRS:** competitive with LRU but never beat W-TinyLFU. HIR share is a trade between traces. Dropped.\n- **Ghost-list SLRU:** promoting ghost hits into protected hurt. Ghost signals are acceptable only as inputs to the admission or split comparison.\n"
}
```

### 12:51:47 thinking



### 12:51:47 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output dir. It has the four required sections in order, built only from the ledger digest of cycles 1–10.

- **Recurring wins:** It traces the line of cycle winners: LRU, then SLRU, then W-TinyLFU over SLRU, then the tightened sketch, capped climber, size-gated step and split, and longer small-size sketch memory. Lessons that held across cycles: soft, relative signals beat hard overrides; strict one-shot filtering carries hm_0; changing one lever per variant makes results traceable; size gates are reliable; fixed caps beat capacity-scaled settings.
- **Recurring failures:** Loosening admission, adding a bonus for in-window reuse (the sketch already counts those hits), climber tuning, draining protected, large-size static gates (all trade web_0 against hm_0), sketch aging past its peak, doorkeeper resizing and LIRS. `metadata-size`, `per-op-time` and `win_cap` have been missing for all 10 cycles, so no cost claim has been checked.
- **Open questions:** A strict-beat tie rule for warm victims, limiting conservative update to small size, a separate clear schedule for the doorkeeper, a shorter large-size `reset_at`, and a ghost-hit or shadow signal for adapting the split at large size.
- **Per-policy notes:** Short status for LRU, SLRU, CLOCK, TinyLFU, W-TinyLFU (the current line, with its settings to keep), LIRS and ghost-list SLRU.

The file gives no numbers, only qualitative descriptions of each dimension.


---

- finished: 2026-10-28T12:51:48+00:00
- duration: 28.21s
- status: ok
- input_tokens: 4
- output_tokens: 2577
- cache_read_tokens: 44083
- cache_write_tokens: 16155
- cost_usd: 0.189613
