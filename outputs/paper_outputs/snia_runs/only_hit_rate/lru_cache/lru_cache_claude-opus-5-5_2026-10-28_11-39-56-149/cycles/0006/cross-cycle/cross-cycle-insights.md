# Cross-Cycle Insights (through cycle 0006)

## Recurring wins

- **Soft, relative filtering beats hard gates** (cycles 1–4). SLRU probation, a duel against the current victim that admits ties, and capped adaptation all helped. Strict rejects, full-protected promotion and unconditional bypasses all hurt.
- **The lineage that keeps paying off is LRU → SLRU → W-TinyLFU → tightened sketch → capped hill-climbed window.** Each step to cycle 0004 was the only Winner of its cycle. Cycle 0004's v002 is still the effective parent and the first variant to improve every cell.
- **Strict one-shot filtering drives the hm_0 gains**, most of all at small size. These parts carry the load: dedup on miss-then-put, a wide sketch, the doorkeeper and the long reset period (cycles 3–4).
- **Small-size web_0 responds only to a larger front segment** (window or HIR queue). Every recovery of that cell came from an adaptive window that grew (cycles 3–4).
- **Absolute caps beat parameters that scale with capacity.** The climber stopped losing at large size once its step and period were capped instead of scaled (cycle 4).
- **Changing one lever per variant makes results attributable** (cycles 4–6). Bundled changes (cycle 3) left causes unclear.

## Recurring failures

- **Fixed fractions tied to capacity fail at one end of the size range.** A 1% window or HIR share is too narrow at small size. A ghost list as large as capacity hurts at large size. Scaled climber knobs lost at large size (cycles 2–3).
- **Loosening admission leaks the small-size hm_0 gain.** Shorter resets, doorkeeper clears, in-window bypasses and duel bonuses all did this (cycles 4–5).
- **In-window reuse signals count the same evidence twice.** The sketch already counts those hits. Both the bypass (cycle 4) and the bounded bonus (cycle 5) lost.
- **Climber tuning is used up** (cycles 5–6). Raising the ceiling, shorter or longer periods, a larger step and flip hysteresis were each neutral or worse. The cycle 0004 settings are a local optimum. The two latest Winners won only because they changed nothing for the worse.
- **LIRS** (cycles 2–3) beat LRU but never beat the W-TinyLFU line. Tuning its HIR share only trades hm_0 against web_0.
- **CLOCK** (cycle 1) decays toward FIFO at large size on web_0.
- **Cost axes were never measured.** `metadata-size` and `per-op-time` are missing in all six cycles, so every cost claim is unverified. The climber's `win_cap` was never read either.

## Open questions

- What limits small-size web_0 now? Candidates: the doorkeeper's cold first-touch estimates, or the probation/protected split of the main area. Neither has been tested in isolation under the adaptive window.
- Where does `win_cap` settle in each cell, and does it oscillate at small size? A cycle 0006 v001 debug log may exist.
- Would a step scaled to capacity at large size only (about 1% or less, with the period scaled to match) help large-size web_0 without touching small size?
- Which climber cap does the work: the step cap or the period cap? The two were never separated.
- How do sketch, doorkeeper, window and climber compare with plain SLRU on metadata size and per-op time?

## Per-policy notes

- **LRU (baseline):** every structured variant beat it on hm_0. web_0 favors its recency at small size.
- **SLRU:** a solid base that gains more as capacity grows. Ghost-to-protected promotion hurt. If ghosts are used again, they should feed only the admission comparison. The probation/protected split has not been tuned since cycle 1.
- **CLOCK:** possibly cheaper on cost axes, but unmeasured. Its hit-rate slipped slightly at large size on web_0. CLOCK-based segmentation is untested.
- **TinyLFU over plain LRU:** a hard strict-greater gate hurts recency traces. Do not retry it.
- **W-TinyLFU over SLRU:** the current lineage. Keep tie-admission, the long reset, the doorkeeper, dedup, the wide sketch, the capped climber and the window floor. Further work should target admission and the main area, not the climber.
- **LIRS:** competitive but stuck in a trace-dependent trade-off. Set aside.
