# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:20:57+00:00

## timeline

### 12:21:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0006)\n\n## Recurring wins\n\n- **Soft, relative filtering beats hard gates** (cycles 1–4). SLRU probation, a duel against the current victim that admits ties, and capped adaptation all helped. Strict rejects, full-protected promotion and unconditional bypasses all hurt.\n- **The lineage that keeps paying off is LRU → SLRU → W-TinyLFU → tightened sketch → capped hill-climbed window.** Each step to cycle 0004 was the only Winner of its cycle. Cycle 0004's v002 is still the effective parent and the first variant to improve every cell.\n- **Strict one-shot filtering drives the hm_0 gains**, most of all at small size. These parts carry the load: dedup on miss-then-put, a wide sketch, the doorkeeper and the long reset period (cycles 3–4).\n- **Small-size web_0 responds only to a larger front segment** (window or HIR queue). Every recovery of that cell came from an adaptive window that grew (cycles 3–4).\n- **Absolute caps beat parameters that scale with capacity.** The climber stopped losing at large size once its step and period were capped instead of scaled (cycle 4).\n- **Changing one lever per variant makes results attributable** (cycles 4–6). Bundled changes (cycle 3) left causes unclear.\n\n## Recurring failures\n\n- **Fixed fractions tied to capacity fail at one end of the size range.** A 1% window or HIR share is too narrow at small size. A ghost list as large as capacity hurts at large size. Scaled climber knobs lost at large size (cycles 2–3).\n- **Loosening admission leaks the small-size hm_0 gain.** Shorter resets, doorkeeper clears, in-window bypasses and duel bonuses all did this (cycles 4–5).\n- **In-window reuse signals count the same evidence twice.** The sketch already counts those hits. Both the bypass (cycle 4) and the bounded bonus (cycle 5) lost.\n- **Climber tuning is used up** (cycles 5–6). Raising the ceiling, shorter or longer periods, a larger step and flip hysteresis were each neutral or worse. The cycle 0004 settings are a local optimum. The two latest Winners won only because they changed nothing for the worse.\n- **LIRS** (cycles 2–3) beat LRU but never beat the W-TinyLFU line. Tuning its HIR share only trades hm_0 against web_0.\n- **CLOCK** (cycle 1) decays toward FIFO at large size on web_0.\n- **Cost axes were never measured.** `metadata-size` and `per-op-time` are missing in all six cycles, so every cost claim is unverified. The climber's `win_cap` was never read either.\n\n## Open questions\n\n- What limits small-size web_0 now? Candidates: the doorkeeper's cold first-touch estimates, or the probation/protected split of the main area. Neither has been tested in isolation under the adaptive window.\n- Where does `win_cap` settle in each cell, and does it oscillate at small size? A cycle 0006 v001 debug log may exist.\n- Would a step scaled to capacity at large size only (about 1% or less, with the period scaled to match) help large-size web_0 without touching small size?\n- Which climber cap does the work: the step cap or the period cap? The two were never separated.\n- How do sketch, doorkeeper, window and climber compare with plain SLRU on metadata size and per-op time?\n\n## Per-policy notes\n\n- **LRU (baseline):** every structured variant beat it on hm_0. web_0 favors its recency at small size.\n- **SLRU:** a solid base that gains more as capacity grows. Ghost-to-protected promotion hurt. If ghosts are used again, they should feed only the admission comparison. The probation/protected split has not been tuned since cycle 1.\n- **CLOCK:** possibly cheaper on cost axes, but unmeasured. Its hit-rate slipped slightly at large size on web_0. CLOCK-based segmentation is untested.\n- **TinyLFU over plain LRU:** a hard strict-greater gate hurts recency traces. Do not retry it.\n- **W-TinyLFU over SLRU:** the current lineage. Keep tie-admission, the long reset, the doorkeeper, dedup, the wide sketch, the capped climber and the window floor. Further work should target admission and the main area, not the climber.\n- **LIRS:** competitive but stuck in a trace-dependent trade-off. Set aside.\n"
}
```

### 12:21:20 thinking



### 12:21:20 chat

I wrote `cross-cycle-insights.md` from the ledger for cycles 0001–0006, using the four required sections in order.

The main points it carries forward:

- **Best line so far:** cycle 0004's v002 is still the effective parent. It is W-TinyLFU over SLRU, with the tightened sketch and the capped hill-climbed window. It is the only variant that improved every trace and size.
- **What keeps working:** soft, relative admission beats hard gates. Strict one-shot filtering drives the hm_0 gains. Small-size web_0 only moves up when the front window gets bigger.
- **What keeps failing:**
  - Settings tied to a fixed fraction of capacity.
  - Anything that loosens admission.
  - Bonuses or bypasses for reuse inside the window, which count evidence the sketch already has.
  - More climber tuning: cycles 0005 and 0006 produced no real gains.
- **Never measured:** `metadata-size`, `per-op-time` and the climber's settled window size have not been reported in any of the six cycles. Every claim about cost is still unverified.
- **Next to test:** whether the doorkeeper or the main area's probation/protected split is now what limits small-size web_0.

The per-policy notes conclude that plain TinyLFU over LRU should not be retried, LIRS should be set aside, and CLOCK is only worth keeping if the cost axes ever get measured.


---

- finished: 2026-10-28T12:21:21+00:00
- duration: 23.72s
- status: ok
- input_tokens: 4
- output_tokens: 2157
- cache_read_tokens: 40145
- cache_write_tokens: 11884
- cost_usd: 0.146257
