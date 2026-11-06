# Variant v001 Insight

## Helped

- hit-rate: hm_0 small improved very slightly. The tie rule still does almost nothing on this cell. Window hits also bump the sketch, so reused candidates usually win under strict `>` before the tie gate matters. Allowing more of them to win ties made little difference.

## Hurt

- hit-rate: web_0 small got worse, not better, which is the reverse of what the hypothesis predicted. Lowering `LATE_HIT_DIVISOR` to 2 lets candidates whose only window hit came earlier in their residency win estimate ties. Those mid-residency hits did not predict reuse in probation. The extra admissions pushed out probation entries that were tied with them but more useful.
- hit-rate: web_0 large lost part of the recovery the parent made. With the long window at 131072, a "last half" cutoff readmits many candidates that the "last quarter" rule had correctly sent back to strict `>`. This partly brings back the drift that the sticky reuse flag caused for long-resident candidates in c8 v003.
- hit-rate: hm_0 large was essentially flat, so the looser gate did not buy anything there to offset the web_0 losses.

## Takeaway

Loosening the late-hit cutoff from 1/4 to 1/2 of window residency hurt web_0 at both cache sizes. So the "last quarter" rule was not too strict for short windows. Extra tie wins in web_0 replace better probation entries at any window length, and the gap at web_0 small that the parent left versus c8 v003 needs a lever other than this cutoff. The hypothesis's own fallback plan, a window-scaled cutoff, rested on the large cells falling while the small cell recovered. That did not happen, because the small cell fell too, so the fallback is not supported either. The cutoff should stay at 1/4 or tighten. A smaller tie gate is likely to help more than a looser one.
