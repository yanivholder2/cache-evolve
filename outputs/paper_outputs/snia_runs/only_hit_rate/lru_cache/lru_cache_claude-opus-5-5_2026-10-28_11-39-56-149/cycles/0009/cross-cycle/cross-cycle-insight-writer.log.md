# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:43:54+00:00

## timeline

### 12:44:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Soft, relative filtering beats hard gates. SLRU probation (c1), a W-TinyLFU duel that admits ties (c2) and strict one-shot filtering (dedup + doorkeeper, c3) all paid off. Each one compares a candidate against the current victim and avoids absolute overrides.\n- Strict one-shot filtering and a large protected segment carry the hm_0 gains at both sizes. The long reset period, the doorkeeper, dedup and the wide sketch have all been confirmed as necessary (c3–c4, c7).\n- An adaptive window whose climber knobs are capped at absolute values rather than scaled with capacity (c4) was the only change to improve every cell. A window floor at the old size protected hm_0.\n- Changing one lever per variant makes results attributable. Size gating (32_768) has held in every cycle since c7: cells on the far side of a gate stay exactly unchanged.\n- Small static large-size gates gave modest net gains on large-size web_0, each costing a little large-size hm_0: a larger initial step (c7) and a 75% protected split (c8).\n- Longer sketch memory at small size (32 × `reset_at`, c9) is the first lever that lifted both small traces together. Small-size web_0 had stopped moving after c4.\n\n## Recurring failures\n\n- Loosening admission has lost in three cycles: plain TinyLFU's strict reject on web_0 (c1), ghost-to-protected promotion (c2), an unconditional window bypass (c4), an in-window reuse bonus (c5) and random admission of warm keys (c7). In-window reuse adds nothing beyond the sketch; using it counts the same evidence twice.\n- Faster sketch aging (shorter reset, c4) hurts small-size hm_0. The evidence consistently favours longer memory.\n- Fixed capacity-proportional fractions (a 1% window or HIR share, a ghost list sized to capacity, climber knobs scaled with capacity) fail at one end of the size range (c2–c3).\n- Climber tuning is used up. The ceiling, a shorter or longer period, a larger step cap, two-drop hysteresis and flip damping were all neutral or worse (c5–c8). The c4 settings are a local optimum.\n- Global changes to the main-area split, and anything that drains protected faster, hurt both traces. That includes a smaller protected share (c7), a frequency-based victim choice between probation and protected (c9) and a non-decaying hit-rate climber on the split (c9).\n- Making the doorkeeper more accurate (c8) removed its uniform first-touch bonus and hurt small-size hm_0.\n- LIRS and CLOCK never beat the SLRU/W-TinyLFU line. Tuning LIRS's HIR share just trades one trace for the other.\n\n## Open questions\n\n- How far can small-size `reset_at` go (48–64 ×) before stale counts lock out new hot keys? Does a longer `reset_at` help at large size only?\n- Is there a clean runtime signal (ghost hits, a shadow cache, the hit-rate response after a flip) that can tell scan-heavy traffic from medium-reuse traffic, so the large-size protected split could adapt without bias?\n- Where does `win_cap` settle in each cell? It has never been read.\n- Do `metadata-size` and `per-op-time` change any conclusion? Neither has been measured in nine cycles, so every cost claim (CLOCK, sketch, ghost lists, climber sampling) is unverified.\n\n## Per-policy notes\n\n- LRU baseline: beaten by every SLRU-based design. Plain TinyLFU in front of LRU lost on recency-heavy web_0.\n- SLRU: a solid base. A large protected share is necessary. A longer probation helps only medium-range reuse at large size and costs hm_0.\n- W-TinyLFU over SLRU: the current lineage. Parent is c9 v002: tightened sketch (long reset, doorkeeper, dedup, wide), capped climber with a larger initial step at large size, 75% protected at large size and 80% at small, 32 × `reset_at` at small size.\n- CLOCK: drifts toward FIFO at large ring sizes on web_0. Its supposed cost advantage has never been measured.\n- LIRS: competitive with LRU but always behind W-TinyLFU. Shelved.\n- Traces: hm_0 rewards frequency and one-shot filtering. Small-size hm_0 is the most fragile cell. web_0 rewards recency and window size. Small-size web_0 is the hardest cell to move and responded only to window growth (c4) and longer sketch memory (c9).\n"
}
```

### 12:44:22 thinking



### 12:44:22 chat

I wrote `cross-cycle-insights.md` with the four required sections in order. I built it only from the ledger for cycles 0001–0009.

The main points it records:

- **What keeps working:**
  - **Gentle filtering:** filters that compare a new block against the one it would evict, and admit on ties, beat hard reject rules.
  - **Two things hm_0 depends on:** filtering out blocks seen only once, and keeping a large protected segment.
  - **Window cycles:** a window sized by a climber with fixed caps (cycle 4) improved every cell.
  - **Method:** changing one lever per variant, behind a cache-size gate, keeps results clean to attribute.
  - **Small-size breakthrough:** a longer sketch memory at small size (cycle 9) is the first change in many cycles to lift small-size web_0, and it helped hm_0 there too.
- **What keeps failing:**
  - **Looser admission** (bypasses, bonuses, random admission) lost in every form tried.
  - **Faster aging** of the frequency counts hurt.
  - **Fixed fractions tied to capacity** fail at one end of the size range.
  - **Climber tuning:** every knob has been tried and none helps any more.
  - **Shrinking the protected segment** hurt both traces.
  - **A more accurate doorkeeper** hurt small-size hm_0.
  - **LIRS and CLOCK** never beat the SLRU / W-TinyLFU line.
- **Open questions:**
  - How far the small-size sketch memory can be pushed, and whether it helps at large size.
  - Whether some live signal is clean enough to adapt the protected split at large size.
  - Where the climber's window size actually settles; that log has never been read.
  - The cost measures (memory overhead and time per operation): they have been missing for all nine cycles, so every cost claim is still unverified.
- **Per-policy notes:** these cover LRU, SLRU, W-TinyLFU (including the current parent, cycle 9's v002), CLOCK and LIRS, plus how the two traces differ.


---

- finished: 2026-10-28T12:44:23+00:00
- duration: 29.13s
- status: ok
- input_tokens: 4
- output_tokens: 2393
- cache_read_tokens: 43018
- cache_write_tokens: 14777
- cost_usd: 0.174696
