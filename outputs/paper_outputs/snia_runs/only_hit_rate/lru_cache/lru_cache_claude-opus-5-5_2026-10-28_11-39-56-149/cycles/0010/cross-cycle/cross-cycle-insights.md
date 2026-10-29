# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- **Lineage:** LRU → SLRU (c1) → W-TinyLFU over SLRU with tie-admitting duel (c2) → tightened sketch: dedup, wide, doorkeeper, long reset (c3) → capped hill-climbed window with floor (c4) → large-only initial step (c7) → large-only 75% protected split (c8) → small-only 32× `reset_at` (c9) → conservative update (c10, fragile). Every real gain was an extension of this line.
- **Soft, relative signals beat hard overrides** (c1, c2, c4, c5): the duel against the current victim with ties admitted works. Hard gates, bypasses and full-status promotion all lost.
- **Strict one-shot filtering is what wins hm_0** (c1, c3, c4, c7): dedup, doorkeeper and the long reset all carry real weight. Small-size hm_0 is the most fragile cell.
- **Enlarging the front window is the main lever for recency-heavy web_0** (c3, c4). The capped adaptive climber gave the run's only all-cell gain.
- **Changing one lever per variant makes results attributable** (c4 onward). Bundled changes (c3) could not be diagnosed.
- **Size gates are reliable** (c7, c8, c9): cells on the other side of the gate stay exactly unchanged, so you can tune per size without collateral effects.
- **Capping parameters at absolute values works better than scaling them with capacity** (c2, c3, c4). Settings tied to capacity keep failing at one end of the size range.
- **Longer sketch memory helped at small size** (c9): the first lever that lifted both small traces together. Its benefit has now peaked (c10).

## Recurring failures

- **Loosening admission** (c1 hard gate aside, c4 bypass, c5 window-reuse bonus, c7 random admission): this lost every time, and hm_0 paid first. The tie-admitting duel is already lenient enough.
- **Counting the same evidence twice:** in-window reuse is already in the sketch, so bonuses or bypasses based on it skew close duels toward recency (c4, c5).
- **Ghost or non-resident history used as promotion** (c2): one return from outside the cache is weak evidence on web_0.
- **Tuning the climber** (c5, c6): the ceiling, shorter or longer periods, a bigger step cap, hysteresis and flip damping (c8) were all neutral or worse. The climber is at a local optimum.
- **Draining protected faster** (c7 global split, c9 frequency-based victim, c9 split climber): protected residency carries weight for both traces at large size.
- **Large-size static gates trade web_0 against hm_0** (c7, c8, c10). Step, split, damping and longer memory all move along that same trade-off. No lever has helped both large traces together.
- **Faster sketch aging** (c4) and **longer aging past its peak** (c10, both sizes) both hurt.
- **Resizing the doorkeeper** (c8): a more accurate doorkeeper removes the uniform first-touch bonus and hurts hm_0.
- **LIRS** (c2, c3): tuning its HIR share only moves hit-rate between traces. Dropped.
- **Measurement debt:** `metadata-size`, `per-op-time` and `win_cap` have not been reported in any of the 10 cycles. Every cost claim is unverified.

## Open questions

- Does strict-beat for warm victims (count > 1) keep c10's web_0 gain and recover hm_0? Tie strictness is the hidden hinge behind several results (c1, c2, c10).
- Should conservative update be gated to small size only? Its large-size cells got nothing or lost.
- Doorkeeper fill and `reset_at` are linked (c8, c10). Would clearing the doorkeeper on its own shorter schedule let long memory stop leaking +1 bonuses to one-shot keys?
- At large size, is a shorter `reset_at` (around 8×) the right direction?
- Can a clean workload signal (a ghost hit or a shadow comparison) drive protected-split adaptation at large size, where raw hit-rate climbers failed?
- Where does `win_cap` settle in each cell, and what do the sketch, window and climber cost per op and in metadata?

## Per-policy notes

- **LRU (baseline):** beaten by every later design. Use it only as the reference point.
- **SLRU:** a soft probation filter that helps more as capacity grows. The large protected segment is the critical part, and 80% small / 75% large is the current split. Global split changes and frequency-based victim rules hurt.
- **CLOCK:** decayed toward FIFO on web_0 at large size. Its possible cost advantage was never measured. Dormant.
- **TinyLFU (plain over LRU):** a strict-greater reject hurts recency traces. Do not retry.
- **W-TinyLFU over SLRU (current line):** keep the tie-admitting duel, dedup, wide sketch, doorkeeper, long reset (32× small, 16× large), capped climber with floor, and large-only step and split gates. Conservative update is the newest addition and is fragile.
- **LIRS:** competitive with LRU but never beat W-TinyLFU. HIR share is a trade between traces. Dropped.
- **Ghost-list SLRU:** promoting ghost hits into protected hurt. Ghost signals are acceptable only as inputs to the admission or split comparison.
