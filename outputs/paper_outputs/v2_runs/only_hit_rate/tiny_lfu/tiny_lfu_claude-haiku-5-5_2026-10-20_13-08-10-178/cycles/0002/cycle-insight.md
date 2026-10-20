# Cycle 0002 Insight

## Cohort outcome

The cohort tried two layout-only changes to the frequency sketch's storage (narrower counters, and a flattened row-major counter array) and one structural change adding a small recency window ahead of the main LRU with admission applied at window exit. Both Winners, v001 and v003, were layout changes that left hit-rate flat; the window variant, v002, lost a little hit-rate and did not advance.

## Patterns

- Layout-only changes to the sketch were the only ones that held hit-rate. Both Winners preserve the logical counter mapping, so their hit-rate is unchanged by construction rather than by a measured improvement.
- Changing when and where admission sees a new key cost hit-rate on this trace; the window's benefit for one-touch bursts was not visible.
- None of the cohort recorded a hit-rate gain. Every benefit claimed by the Winners (memory footprint, per-op cost) sits on dimensions the fitness vector does not measure, so the Winners advance by not regressing.
- The unmeasured-dimension problem carried over from Cycle 0001 is unresolved: layout work keeps producing Winners whose payoff cannot be confirmed.

## Open questions

- Would a memory or per-operation timing dimension in the fitness vector confirm the layout gains, and do they justify keeping both Winners?
- Does the narrower counter type ever create admission ties on traces with very hot keys, where saturation could matter?
- Is there a window or admission structure that helps one-touch bursts without giving up the sketch signal at this capacity?

## Continuation

Next cycle, keep to changes that preserve sketch counts and the admission rule, and prefer layout or allocation work only if the fitness vector gains a metadata or timing signal that can confirm it. Drop the recency-window-in-front-of-LRU structure at this capacity, and do not stack further layout changes until the Winners' gains are measured.
