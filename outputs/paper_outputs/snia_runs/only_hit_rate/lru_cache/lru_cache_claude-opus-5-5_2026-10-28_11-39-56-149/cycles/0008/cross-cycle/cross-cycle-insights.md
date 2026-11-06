# Cross-Cycle Insights

## Recurring wins

- Filtering out blocks seen only once is the idea behind every winner in the lineage: SLRU (c1), then W-TinyLFU over SLRU (c2), then a stricter one-shot filter (c3). Scan-heavy hm_0 rewards it every time.
- Soft, relative signals beat hard overrides. The tie-admitting duel against the current victim has won since c2. Hard gates, direct promotion and bypasses all lost (c1, c2, c4).
- Enlarging the window was the only lever that moved small-size web_0 by a large amount. The hill-climbed window with absolute caps and a floor at the old size (c4) is the single biggest gain in the run.
- Changing one lever per variant made results attributable (c4 onward). Bundled changes in c3 could not be diagnosed.
- Gating parameters by cache size is reliable. Size-gated changes left the other side of the gate exactly unchanged (c7, c8). The large-only larger step (c7) and the large-only 75% protected split (c8) each gave small strict gains.
- These parts carry weight and should be kept: the long sketch reset, the doorkeeper, dedup on miss-then-put, the wide sketch, a large protected segment (at least at small size), and the deterministic tie-admitting duel.

## Recurring failures

- Loosening admission. Hard TinyLFU reject was too strict (c1), but every loosening since then has lost, and small-size hm_0 lost first: in-window bypass (c4), reuse bonus (c5), random admission of warm keys (c7). Admission is not too strict.
- Counting in-window reuse again. Window hits are already in the sketch, so any bonus double-counts them and tilts close duels toward recency.
- Fixed fractions or parameters scaled with capacity tend to fail at one end of the size range: ghost list (c2), 1% front segment (c2), linearly scaled climber knobs (c3).
- Climber tuning is used up. Ceiling, shorter or longer period, larger step cap, two-drop hysteresis and flip damping were all neutral or worse (c5–c8). Single-sample drops are real signals, so delaying reactions hurts.
- Faster sketch aging (c4) and a more accurate doorkeeper (c8) both hurt small-size hm_0 and did not help web_0.
- Ghost-to-protected promotion and LIRS (any HIR share) gave trace-dependent trades, not net gains.
- Shrinking protected globally pushes hm_0's reuse set into the eviction path (c7).
- Every cycle so far has reported hit-rate only. `metadata-size`, `per-op-time` and `win_cap` logs have never been read, so the cost claims are unverified for every policy.

## Open questions

- What limits small-size web_0? It has resisted climber, split, admission-loosening and doorkeeper changes. Untested small-only levers: the duel tie-break, the `reset_at` period, and the window/main split.
- At large size, static gates now only move hit-rate between web_0 and hm_0. Can a runtime signal (response after a flip, how scan-heavy the window traffic is, an adaptive protected split) separate the two workloads?
- Where does `win_cap` settle in each cell, and does it oscillate at small size?
- What are the actual metadata and per-op costs of the sketch, doorkeeper and climber compared with plain SLRU or CLOCK?

## Per-policy notes

- **LRU (baseline):** beaten in nearly every cell by every segmented or filtered design.
- **SLRU:** the core of every winner. Probation shields hm_0 from one-shot blocks. A longer probation helps large-size web_0 but costs hm_0 unless it is gated to large size only.
- **CLOCK:** decays toward FIFO with a large ring on web_0. It was kept as a cost-axis candidate but never re-tested, because costs were never measured.
- **TinyLFU / W-TinyLFU:** plain TinyLFU over LRU hurt recency traffic. W-TinyLFU with a tie-admitting duel over SLRU, an adaptive capped window, and a strict sketch (long reset, doorkeeper, dedup, wide) is the current lineage. Current parent: c8 v002 (75% large-size protected split, 80% at small size, large-only step).
- **LIRS:** competitive with LRU but never beat W-TinyLFU. Tuning its HIR share swaps one trace for the other. Dropped.
- **Ghost-list SLRU:** ghost hits are a weak signal on web_0. They may help only as input to the admission comparison, never as direct promotion.
