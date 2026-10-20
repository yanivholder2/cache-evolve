# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_13-08-10-178/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried one sketch-decay change, one layout-only narrowing of node and index types, and one change to how the sketch derives counter positions from a key. The only Winner, v002, was the layout-only change, which left admission and the sketch untouched; the two variants that altered sketch behaviour did not hold up on hit-rate.

## Patterns

- Changes that touched the frequency sketch or the admission comparison were the ones that lost hit-rate; both sketch-side variants moved admission estimates in the wrong direction.
- Decay of the sketch on a fixed schedule discarded signal that admission needed at this window size, rather than removing stale history.
- The Winner's payoff is not yet demonstrated: its fitness reflects only hit-rate, which it kept flat, so its intended metadata-size gain is unmeasured.
- None of the variant insights recorded a hit-rate gain, so this cycle's Winner advanced by not regressing rather than by improving.

## Open questions

- Does the per-entry size reduction from the Winner produce a measurable benefit once a metadata or memory dimension is part of the fitness vector?
- Is the small hit-rate dip from the key-to-counter mapping change within noise, or a real regression that should rule out that family of changes?
- Would a gentler or staleness-gated decay of the sketch recover signal without the admission losses seen here?

## Continuation

Next cycle, pursue changes that leave the sketch and admission rule intact and target dimensions the fitness vector actually measures, and add a measurable metadata or memory signal before judging layout work. Drop plain fixed-schedule sketch halving at this window size, and do not re-run key-hashing rewrites unless they are shown to be within noise.


### Cycle 0002

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
