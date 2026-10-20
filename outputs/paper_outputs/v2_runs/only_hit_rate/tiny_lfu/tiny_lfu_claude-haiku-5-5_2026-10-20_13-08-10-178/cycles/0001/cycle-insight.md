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
