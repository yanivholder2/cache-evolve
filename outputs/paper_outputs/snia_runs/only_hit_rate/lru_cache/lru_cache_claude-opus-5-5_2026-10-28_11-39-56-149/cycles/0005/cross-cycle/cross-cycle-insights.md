# Cross-Cycle Insights

## Recurring wins

- Keeping blocks seen only once from displacing reused blocks is the idea behind every Winner so far: SLRU, then W-TinyLFU over SLRU, then its tightened sketch, then its capped climber. The lineage has improved each time it was built on.
- Soft, relative signals beat hard overrides. Probation lists, a victim-vs-candidate duel with ties admitted, and gradual window adaptation help. Hard gates, ghost-to-protected promotion and unconditional bypasses hurt.
- Strict one-shot filtering drives the hm_0 gains: dedup on miss-then-put, a wide sketch, a doorkeeper and a long reset period. These are load-bearing, especially at small capacity.
- Enlarging the front (recency) segment is the only lever that has really moved small-size web_0. The adaptive window with absolute caps (cycle 0004) is the only change that improved every cell.
- Changing one lever per variant has made results attributable since cycle 0004. Bundled changes (cycles 0002–0003) left causes unclear.
- Setting parameters as absolute values or capping them, instead of scaling them linearly with capacity, avoids losses at one end of the size range.

## Recurring failures

- Fixed capacity-proportional fractions (a 1% window or HIR share, a capacity-sized ghost list, a climber step or period scaled with capacity) fail at either the small or the large size.
- Loosening admission to help web_0 leaks hm_0 gains every time. This covered a strict-reject TinyLFU, shorter reset periods, an in-window bypass and an in-window duel bonus.
- In-window reuse as an extra signal fails because the sketch already counts it, so it double-counts. Both the hard bypass and the soft bonus lost.
- Speeding up the climber (shorter sample-period cap) measures half-filled resizes and reverses growth that would have paid off.
- Raising the window ceiling does nothing, because the window never reaches the old 40% limit.
- LIRS, including the tuned HIR share, only trades hm_0 against web_0 and never beat the W-TinyLFU line.
- Process failure: `metadata-size` and `per-op-time` have gone unmeasured for five cycles, so every cost claim (CLOCK's cheap `get`, sketch, ghost and climber overhead) is still unverified. The climber's window size per cell has also not been read.

## Open questions

- Where does the adaptive window settle in each cell, and does it oscillate at small size?
- Does a larger step cap, with the period cap held fixed, grow the window further on small-size web_0 without hurting hm_0?
- Would a sample period tied to fill time or capacity, or a less sensitive flip-on-drop (for example, two drops in a row), beat the fixed-count period?
- With the adaptive window in place, does the doorkeeper still cost anything on web_0?
- What do the current champion's sketch, window and climber cost on the cost axes compared with plain SLRU or LRU? Would a CLOCK-based segmented variant keep the hit-rate at lower cost?
- Can any signal that does not depend on the sketch (as in-window reuse does) help close duels on web_0?

## Per-policy notes

- **LRU (baseline):** beaten by every later design. web_0 at small size relies on short-range recency reuse that LRU already captures fairly well.
- **SLRU:** the base layer of every Winner. Gains grew with capacity. Its probation list is too short for hm_0's long reuse distances unless something sits in front of it.
- **CLOCK:** helped modestly but decayed toward FIFO on web_0 at large size. It was kept only for possible cost-axis benefits, which have never been measured.
- **TinyLFU over LRU (strict reject):** hurt web_0, worst at small size. Do not retry.
- **SLRU + ghost list:** sending ghost hits straight to protected hurt web_0. Ghost history should feed admission at most.
- **W-TinyLFU over SLRU:** the current lineage and champion. The effective parent is cycle 0004's v002: a long reset, doorkeeper, dedup and wide sketch; a hill-climbed window with absolute step and period caps and a floor at the old window size; and a tie-admitting duel. Cycle 0005's v003 matches it in every cell, with a harmless higher ceiling.
- **LIRS:** competitive with LRU, but its weak cell is the same as W-TinyLFU's and its HIR-share tuning is a trade between traces. Shelved.
