## Recurring wins

- Changes that keep the sketch's counts and the admission rule intact, and reorder recency inside the main region, are the lever that has produced a hit-rate gain (probation/protected split, Cycle 0003).
- Widening the frequency sketch to cut over-counting from colliding keys is the one sketch-side change that has moved hit-rate up; sketch changes aimed at collision error look more promising than those aimed at decay or key mapping (Cycle 0004).
- Layout-only sketch changes reliably hold hit-rate flat, so they can advance as non-regressions, but they have not shown a gain on the measured fitness dimension.

## Recurring failures

- Changing when or where admission sees a new key (e.g. a recency window in front of the LRU) costs hit-rate on this trace (Cycle 0002).
- Fixed-schedule halving of the sketch discards history that admission still needs at this capacity (Cycle 0001).
- Key-to-counter remapping produced a small hit-rate dip and has not been shown to be noise (Cycle 0001).
- Main-region variants that shrink the protected segment or push demoted protected entries to the front of probation both lost hit-rate; the current ratio and back-of-probation demotion are the incumbent settings (Cycle 0004).
- Layout Winners justify themselves on footprint or per-op cost, which the fitness vector does not measure, so their payoff stays unconfirmed (Cycles 0001–0003).

## Open questions

- Do the probation/protected and sketch-width gains hold on other workloads and cache sizes? Every result so far comes from one trace at one size.
- What metadata, memory, or timing signal would confirm the layout gains, and is the sketch-width gain worth its metadata cost once measured?
- At what width does further sketch widening stop helping, and does counter saturation ever bind on very hot keys?
- Is the key-remapping dip within noise, or a real regression?
- Would a gentler or staleness-gated sketch decay recover signal without the admission losses seen so far?
- Is there an admission structure that helps one-touch bursts without giving up sketch signal?

## Per-policy notes

- TinyLFU frequency sketch (admission estimate): width increases help; decay on a fixed schedule hurts at this capacity; layout-only storage changes are neutral on hit-rate but unmeasured on cost.
- TinyLFU admission path: changes that alter when or where a new key is considered for admission have lost hit-rate; keep the admission comparison fixed until other changes are settled.
- Main region (SLRU-style probation/protected): the probation/protected split is the best-performing structure so far; the current protected ratio and back-of-probation demotion are the settings to hold, and the split needs validation beyond one trace.
