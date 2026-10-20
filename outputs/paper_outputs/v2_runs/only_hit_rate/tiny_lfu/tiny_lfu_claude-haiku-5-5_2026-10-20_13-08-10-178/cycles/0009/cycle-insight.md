# Cycle 0009 Insight

## Cohort outcome

The cohort tried three unrelated changes: widening the probation victim scan, narrowing the sketch counters to 16 bits, and switching the sketch's halving trigger to a frequency-based rule. The only Winner, v001, widened the victim comparison from two LRU-tail entries to three and evicted the coldest by sketch estimate; its hit-rate gain is very small.

## Patterns

- The victim-selection step is again the only place a gain has appeared. Cycle 0008 found that comparing two tail entries helped, and v001 extends the same rule one step further, so the signal is consistent with the previous cycle but is not yet separated from it.
- Storage narrowing is neutral on hit-rate. The u16 counter change left the fitness vector unchanged, so its only benefit (smaller metadata) is invisible to the current measurement and cannot be credited.
- Global aging hurts. Halving the whole sketch table when one counter saturates erases admission history for cold keys, which breaks the frequency ordering the admission comparison relies on. Aging needs to be scoped per key or per row, not table-wide.
- Every result comes from one trace at one cache size, so the victim-scan gain is provisional.

## Open questions

- Does the victim comparison keep improving past three candidates, or does the gain stop at the current window?
- Does the counter-width change cost hit-rate on a longer or more skewed trace, where 16-bit saturation could start producing ties?
- What per-key or per-row aging rule keeps the admission comparison intact while still bounding counter growth?

## Continuation

Next cycle, confirm the three-entry victim scan on a second workload or cache size before building on it, and treat the victim-selection family as the active thread. Drop the table-wide frequency-triggered halving in its current form. Hold the 16-bit counter change as neutral until a footprint dimension is added to the fitness vector, and do not stack new aging rules on top of it until that measurement exists.
