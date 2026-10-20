# Cycle 0002 Insight

## Cohort outcome

The cohort tried a per-bucket fingerprint filter (v001, the sole Winner), a u8 saturating age counter (v002), and a cache-wide insert-count trigger for the AgedLfu halving (v003). Only hit-rate was measured for every variant, and it did not move in any of them, so the Winner's selection is not explained by the recorded evidence.

## Patterns

- Hit-rate was again the only dimension reported, repeating the Cycle 0001 gap. No variant recorded metadata-size or per-op-time, so the cost-side claims behind all three hypotheses remain untested.
- Hit-rate was flat across the cohort. The v001 insight itself says its fitness vector matches the parent's, which leaves the Winner label unsupported by the data it cites.
- Both aging-related variants (v002, v003) returned no measurable hit-rate gain, a second null result for the aging policy after Cycle 0001's v002. Whether the evaluation actually selects the aging policy is still unresolved.
- The v002 hit-rate drop is described only qualitatively as slight, so the size of any regression cannot be compared across variants.

## Open questions

- Why was v001 marked a Winner when its only measured dimension did not change? Does the evaluation check fingerprint code is in the build it runs?
- Does the AgedLfu halving fire at all under the evaluation harness? How many halvings occur per run?
- Do the fingerprint filter's per-op-time savings and the u8 counter's metadata-size savings show up once those dimensions are measured?

## Continuation

Next cycle should add metadata-size and per-op-time to the fitness vector before judging any cost-motivated change, and should log whether the aging policy is active and how often it halves. Do not pursue further aging-period or counter-width tweaks on hit-rate alone. Keep the fingerprint direction only if a per-op-time measurement is included.
