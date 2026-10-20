# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cycle explored three orthogonal levers: sketch counter aging, a newcomer admission window, and SLRU segmentation of the resident set. Only the SLRU variant (v003) improved hit rate and was selected as the winner; the other two degraded performance.

## Patterns

- **Resident-ordering changes beat admission-policy changes.** The sole winner restructured how residents are ordered and protected after admission; the two losers both intervened at the point of admission (frequency gate timing, sketch accuracy) and both backfired.
- **Capacity cannibalization is immediately punishing.** Reserving a permanent window segment shrank the main LRU and that structural loss was felt before any newcomer-grace benefit could materialize — the trace does not produce items that accumulate frequency fast enough inside the window to recoup the cost.
- **Sketch aging cadence matters as much as the concept.** The aging interval was too short for the workload's hot/cold cycle; counters collapsed faster than the trace refreshes frequency distinctions, making the admission gate noisy rather than adaptive.
- **The frequency gate already handles one-hit-wonders well.** SLRU's protection is partially redundant with the existing admission gate, which explains why its gain was real but narrow.

## Open questions

- Is the SLRU split ratio (80 % protected / 20 % probationary) close to optimal, or does a different ratio extract more gain on this trace?
- At what aging interval (or with what counter floor) does sketch aging shift from harmful to helpful?
- Are there complementary changes to the SLRU structure — such as adaptive segment resizing — that could amplify v003's modest improvement?

## Continuation

Build on v003's SLRU as the new baseline and probe segment-ratio sensitivity (e.g., 90/10 and 70/30 splits) since the 80/20 choice was arbitrary and the win was narrow. Sketch aging remains worth revisiting with a substantially longer reset cadence or a per-key counter floor, but not at the same interval as v001. Deprioritize the admission window: it carries a structural capacity penalty that this trace punishes immediately.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cycle probed three distinct levers — SLRU split-ratio tightening, sketch-aging at a much longer cadence, and a frequency-gated promotion threshold — against the SLRU baseline inherited from cycle 0001. Only the 90/10 split variant won, and only by a negligible margin; the other two produced no improvement or a slight regression.

## Patterns

- **SLRU ratio tuning has reached diminishing returns.** Moving from 80/20 to 90/10 barely moved the needle, confirming that the hot set was already well-contained in the protected segment at 80 %; the extra headroom covers only marginal-frequency items. The shrinkage of the probationary buffer caused no visible harm, reinforcing that probationary is a pass-through staging area on this workload, not a competitive contention zone.
- **Sketch aging is consistently harmful regardless of cadence.** Even at 10× the already-conservative reset interval tested in cycle 0001, periodic halving of counters erodes frequency signal more than it cleans up stale counts. The frequency distribution on this trace is stable across the entire replay window; accumulated counts are signal, not noise.
- **The promotion threshold landed in a behavioral dead zone.** At a threshold of 2, too few borderline items were affected — either they are too rare to matter or they accumulate frequency fast enough to qualify for promotion shortly after anyway. The concept is sound but the threshold value was ineffective at this workload's access rate.
- **All three axes hit a wall together.** None of the three hypotheses produced a meaningful gain, suggesting the SLRU-on-TinyLFU configuration is near a local optimum for workload-a at this cache size, and parameter-level tuning within the current structure is largely exhausted.

## Open questions

- Is there a promotion threshold high enough (e.g., 5–10) to actually reduce protected-segment churn, or does this trace simply not produce enough borderline-frequency contention for the gate to matter?
- What structural changes — beyond SLRU segmentation and sketch aging — could shift hit rate on a stable Zipfian workload? Candidates include ghost/victim caches, adaptive admission biases, or size-aware eviction.
- Is the admission frequency gate itself the tightest bottleneck, or is eviction ordering (within protected) the remaining lever?

## Continuation

SLRU split-ratio tuning and sketch aging are both exhausted as levers for this workload and should be deprioritized in future hypotheses. The next cycle should leave the 90/10 split in place (as the marginal winner) and pivot to structural rather than parametric changes: consider ghost-cache admission feedback (tracking recently evicted keys to bias re-admission), or probing whether the in-segment LRU ordering within protected could be replaced by a frequency-ordered variant. The promotion threshold concept is worth one more test at a substantially higher value, but should not consume a dedicated variant slot unless paired with a complementary change.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
