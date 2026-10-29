# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:14:04+00:00

## timeline

### 12:14:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Keeping blocks seen only once from displacing reused blocks is the idea behind every Winner so far: SLRU, then W-TinyLFU over SLRU, then its tightened sketch, then its capped climber. The lineage has improved each time it was built on.\n- Soft, relative signals beat hard overrides. Probation lists, a victim-vs-candidate duel with ties admitted, and gradual window adaptation help. Hard gates, ghost-to-protected promotion and unconditional bypasses hurt.\n- Strict one-shot filtering drives the hm_0 gains: dedup on miss-then-put, a wide sketch, a doorkeeper and a long reset period. These are load-bearing, especially at small capacity.\n- Enlarging the front (recency) segment is the only lever that has really moved small-size web_0. The adaptive window with absolute caps (cycle 0004) is the only change that improved every cell.\n- Changing one lever per variant has made results attributable since cycle 0004. Bundled changes (cycles 0002–0003) left causes unclear.\n- Setting parameters as absolute values or capping them, instead of scaling them linearly with capacity, avoids losses at one end of the size range.\n\n## Recurring failures\n\n- Fixed capacity-proportional fractions (a 1% window or HIR share, a capacity-sized ghost list, a climber step or period scaled with capacity) fail at either the small or the large size.\n- Loosening admission to help web_0 leaks hm_0 gains every time. This covered a strict-reject TinyLFU, shorter reset periods, an in-window bypass and an in-window duel bonus.\n- In-window reuse as an extra signal fails because the sketch already counts it, so it double-counts. Both the hard bypass and the soft bonus lost.\n- Speeding up the climber (shorter sample-period cap) measures half-filled resizes and reverses growth that would have paid off.\n- Raising the window ceiling does nothing, because the window never reaches the old 40% limit.\n- LIRS, including the tuned HIR share, only trades hm_0 against web_0 and never beat the W-TinyLFU line.\n- Process failure: `metadata-size` and `per-op-time` have gone unmeasured for five cycles, so every cost claim (CLOCK's cheap `get`, sketch, ghost and climber overhead) is still unverified. The climber's window size per cell has also not been read.\n\n## Open questions\n\n- Where does the adaptive window settle in each cell, and does it oscillate at small size?\n- Does a larger step cap, with the period cap held fixed, grow the window further on small-size web_0 without hurting hm_0?\n- Would a sample period tied to fill time or capacity, or a less sensitive flip-on-drop (for example, two drops in a row), beat the fixed-count period?\n- With the adaptive window in place, does the doorkeeper still cost anything on web_0?\n- What do the current champion's sketch, window and climber cost on the cost axes compared with plain SLRU or LRU? Would a CLOCK-based segmented variant keep the hit-rate at lower cost?\n- Can any signal that does not depend on the sketch (as in-window reuse does) help close duels on web_0?\n\n## Per-policy notes\n\n- **LRU (baseline):** beaten by every later design. web_0 at small size relies on short-range recency reuse that LRU already captures fairly well.\n- **SLRU:** the base layer of every Winner. Gains grew with capacity. Its probation list is too short for hm_0's long reuse distances unless something sits in front of it.\n- **CLOCK:** helped modestly but decayed toward FIFO on web_0 at large size. It was kept only for possible cost-axis benefits, which have never been measured.\n- **TinyLFU over LRU (strict reject):** hurt web_0, worst at small size. Do not retry.\n- **SLRU + ghost list:** sending ghost hits straight to protected hurt web_0. Ghost history should feed admission at most.\n- **W-TinyLFU over SLRU:** the current lineage and champion. The effective parent is cycle 0004's v002: a long reset, doorkeeper, dedup and wide sketch; a hill-climbed window with absolute step and period caps and a floor at the old window size; and a tie-admitting duel. Cycle 0005's v003 matches it in every cell, with a harmless higher ceiling.\n- **LIRS:** competitive with LRU, but its weak cell is the same as W-TinyLFU's and its HIR-share tuning is a trade between traces. Shelved.\n"
}
```

### 12:14:28 chat

I've written `cross-cycle-insights.md` to the cycle 0005 cross-cycle folder, using the four required sections in order. I built it only from the five-cycle ledger and didn't read the previous version. The main points:

- **Recurring wins:** Every winner since cycle 0001 extends the same lineage: SLRU, then W-TinyLFU over SLRU, then its tighter sketch, then the capped adaptive window. The best design is still cycle 0004's v002. Cycle 0005's winner matched it in every cell and improved nothing.
- **Recurring failures:**
  - Loosening admission to help web_0 has cost hm_0 gains every time it was tried.
  - Using in-window reuse as an extra signal fails because the sketch already counts it.
  - Fixed settings that scale with capacity fail at either the small or the large size.
  - `metadata-size` and `per-op-time` have gone unmeasured for five cycles, so every cost claim is still unchecked.
- **Open questions:** Where the adaptive window actually settles in each cell has still not been read. Other threads are step-cap tuning, a sample period tied to fill time, and what the current design costs once the cost axes are measured.
- **Per-policy notes:** One short entry each for LRU, SLRU, CLOCK, strict-reject TinyLFU, SLRU with a ghost list, W-TinyLFU and LIRS. Strict-reject TinyLFU and LIRS are marked as not worth retrying.


---

- finished: 2026-10-28T12:14:28+00:00
- duration: 24.46s
- status: ok
- input_tokens: 4
- output_tokens: 2167
- cache_read_tokens: 39203
- cache_write_tokens: 10945
- cost_usd: 0.138757
