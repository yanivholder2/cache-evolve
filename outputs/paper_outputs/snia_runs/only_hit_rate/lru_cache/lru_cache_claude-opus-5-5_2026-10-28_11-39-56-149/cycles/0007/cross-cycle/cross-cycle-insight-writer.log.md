# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:28:25+00:00

## timeline

### 12:28:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Soft, relative filtering beats hard rules. SLRU probation (c1), W-TinyLFU's tie-admitting duel against the current victim (c2), and the tightened sketch (c3) each won. Every hard gate, bypass or override lost.\n- Strict one-shot filtering is the main source of hm_0 gains. Dedup on miss-then-put, the doorkeeper, the long reset period and a large protected segment have all been confirmed as load-bearing (c3, c4, c7).\n- An adaptive front window is the only lever that has meaningfully lifted small-size web_0 (c3 hinted at it; c4 confirmed it with the largest single gain of the run).\n- Scale-aware parameters work. Capping climber knobs at absolute values (c4) and gating changes by cache size (c7 large-only step) removed the losses at one size extreme that capacity-proportional settings kept causing.\n- Changing one lever per variant gives results with a clear cause (c4 onward). Bundled changes (c3) could not be attributed.\n\n## Recurring failures\n\n- Loosening admission has lost in every form tried: unconditional in-window bypass (c4), in-window reuse duel bonus (c5), random admission of warm duel losers (c7), faster sketch aging (c4). Small-size hm_0 always pays the most.\n- In-window reuse as an extra signal counts the sketch's evidence twice and tilts close duels toward recency. It adds no independent information.\n- Promoting ghost or non-resident history straight to protected (c2) overweights a weak signal on web_0.\n- Fixed capacity-proportional fractions (1% window or HIR share, capacity-sized ghost list, climber knobs scaled linearly) fail at one end of the size range (c2, c3).\n- Climber tuning has run out (c5–c7): ceiling, shorter or longer period, larger fixed step, two-drop hysteresis and capacity-tied period were all neutral or worse. Further initial-step increases are also used up.\n- Global main-area split changes (shrinking protected) hurt hm_0 at both sizes (c7).\n- LIRS and HIR-share tuning only trades hm_0 against web_0 (c2, c3).\n- Measurement debt: for seven cycles `metadata-size` and `per-op-time` have never been reported, and `win_cap` has never been read. Cost-axis claims and climber diagnoses remain inferred.\n\n## Open questions\n\n- What still limits small-size web_0? The doorkeeper's cold first-touch estimates is the remaining untested suspect, raised since c4. Test it by disabling or weakening it at small capacity only.\n- Can probation be lengthened at large size only, keeping c7's large-size web_0 signal without exposing hm_0's protected set?\n- Can a separate shrink step, or decay that depends on cache size, recover the small large-size hm_0 loss from the c7 step change?\n- Where does `win_cap` settle in each cell, and does it oscillate at small size?\n- How do the sketch, doorkeeper, window and climber compare with plain SLRU or CLOCK once the cost axes are measured?\n\n## Per-policy notes\n\n- LRU (baseline): beaten in every cycle; serves only as the reference.\n- SLRU: a reliable base (c1 Winner). Gains grow with capacity. A large protected segment is essential for hm_0. Global split changes hurt; large-size-only split changes are untested.\n- CLOCK: slight large-size web_0 loss because it decays toward FIFO when bits stay set. Its expected cost advantage is still unmeasured.\n- TinyLFU over plain LRU: a strict-greater gate hurts recency traffic. Do not retry it.\n- W-TinyLFU over SLRU: the current lineage (c2 → c3 v002 → c4 v002 → c7 v001). Keep: tie-admitting deterministic duel, wide sketch with dedup, doorkeeper, long reset, capped hill-climbed window with floor, large-size-only bigger initial step.\n- Ghost-list SLRU: ghost-to-protected promotion hurt. Ghost history may only be useful as input to the admission comparison.\n- LIRS: competitive with LRU but never beat W-TinyLFU. HIR tuning is a trace trade-off. Shelved.\n"
}
```

### 12:28:52 chat

I wrote `cross-cycle-insights.md` fresh from the seven-cycle ledger. It uses the four required sections, in order.

- **Recurring wins:** Soft, relative admission keeps beating hard rules. Strict filtering of blocks seen only once drives the hm_0 gains. An adaptive front window is the only thing that has really lifted small-size web_0. Capping parameters, or gating them by cache size, works better than scaling them with capacity.
- **Recurring failures:**
  - Every attempt to loosen admission has lost.
  - Climber tuning is used up: cycles 0005–0007 tried every knob without a gain.
  - Changing the probation/protected split for all sizes hurt hm_0.
  - LIRS tuning only swaps hm_0 hit-rate for web_0.
  - For seven cycles, the cost metrics (`metadata-size`, `per-op-time`) and the climber's settled window size (`win_cap`) have never been reported.
- **Open questions:** The top one is whether the doorkeeper limits small-size web_0. It has been suspected since cycle 0004 and never tested directly. The others are a split change at large size only, a separate step for shrinking the window, and where `win_cap` actually settles.
- **Per-policy notes:** These cover LRU, SLRU, CLOCK, TinyLFU, W-TinyLFU (the current line of descent, with the components to keep), the ghost-list variant and LIRS.


---

- finished: 2026-10-28T12:28:52+00:00
- duration: 27.39s
- status: ok
- input_tokens: 4
- output_tokens: 2059
- cache_read_tokens: 41138
- cache_write_tokens: 12780
- cost_usd: 0.151664
