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
