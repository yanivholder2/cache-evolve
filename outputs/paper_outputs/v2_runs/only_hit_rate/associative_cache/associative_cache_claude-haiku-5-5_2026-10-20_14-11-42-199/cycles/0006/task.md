# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried a sampled bucket-victim scan capped at 8 slots (v001), a frequency-gated admission check on inserts (v002), and a two-region bucket split that evicts from the probationary region first (v003). Only v003 improved hit-rate on workload-a and became the Winner; v001 held hit-rate flat and v002 lowered it slightly.

## Patterns

- The one Winner is a scan-resistance change that alters which entries are displaced, not how the existing ranking is computed. This is the first cohort in the digest where hit-rate moved in a positive direction on a mechanism that was actually tested.
- Hit-rate was again the only measured dimension. Per-op time and metadata size were not recorded for any variant, so the cost side of v001 and v003 is still unknown, as in Cycles 0001 and 0002.
- Exact-minimum eviction appears to buy little: capping the victim scan left hit-rate unchanged. Scan cost is therefore a candidate for savings, but only if it is measured.
- The frequency-gated admission in v002 appears to compare estimates on different scales, so its null-to-negative result may reflect a flawed comparison rather than a weak idea.
- Two insights could not confirm from the overlay that the change ran. For v001, an identical hit-rate is also what an unapplied overlay would produce. This leaves the flat result ambiguous.

## Open questions

- Does the sampled-victim change in v001 execute under the harness? An identical hit-rate cannot distinguish "no effect" from "not applied".
- Would a shared frequency counter for both incoming and resident keys make the admission gate in v002 informative?
- What do the second region's metadata and per-op costs in v003 amount to once measured?

## Continuation

Next cycle should build on the scan-resistance direction from v003, and include per-op time and metadata size in the fitness vector before accepting any cost-motivated change. Confirm that each variant's overlay is active before reading a flat hit-rate as a null result. Drop the admission-gate direction unless its two frequency estimates are put on a common scale first.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a second-hit protected tier on top of probationary-first eviction (v001), a sampled probationary victim scan capped at 8 rotating slots (v002), and a single shared frequency counter gating eviction-time admission (v003). Only v003 showed a hit-rate gain on workload-a; v002 was flat to slightly negative and v001 was negative. The recorded Winner is v002, which does not match the hit-rate evidence in its own insight.

## Patterns

- The only positive hit-rate movement came from the shared-frequency admission gate (v003), which compares incoming and resident keys on one scale. This supports the Cycle 0002 suspicion that the earlier admission loss came from a scale mismatch rather than from admission itself.
- The Winner label and the evidence disagree. v002's insight reports a negligible hit-rate dip that misses its own guardrail, while v003 is the variant with a gain. Either the Winner selection used a dimension not shown in the insights, or the label was applied in error. This should be checked before the Winner is treated as a precedent.
- Bounding the victim scan (v002) left hit-rate essentially unchanged, consistent with Cycle 0003's finding that exact-minimum eviction buys little. Its per-op-time benefit remains unmeasured.
- Stacking a second reuse signal (v001) lowered hit-rate. The insight points at the protection cap and demote-on-promotion path rather than the promotion trigger, so the failure is plausibly in the mechanism, not the idea of a second tier.
- No variant measured per-op time or metadata size, and all three insights say the overlays could not be inspected in this run. Flat or negative hit-rate cannot yet be read as a null result about the mechanism, and the cost side is still unknown, as in Cycles 0001 to 0003.

## Open questions

- Which dimension made v002 the Winner, given that v003 carries the only hit-rate gain? Is the Winner rule using a metric missing from the insights?
- Is the v003 gain reproducible across more than workload-a, and does its shared counter add measurable metadata or per-op cost?
- Would a protected tier survive if the cap did not force demotion on each promotion?
- Are the v002 and v001 overlays active under the harness? Without that, their results are ambiguous.

## Continuation

Next cycle should build on the shared-frequency admission gate from v003, and add per-op time and metadata size to the fitness vector before accepting any change. Reconfirm the Winner selection for this cycle before treating v002 as a precedent. Drop the bounded-scan direction as a hit-rate lever, and retry the protected tier only with a changed cap or demotion rule rather than the same promotion path.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a shared-frequency admission gate on top of probationary-first eviction (v001, the Winner), a removal of demote-on-promotion in the protected half-bucket (v002), and inserting misses at the eviction-next position instead of MRU (v003). The two variants that address one-shot inserts both reported a hit-rate gain on workload-a; the protected-tier variant ended slightly below its parent.

## Patterns

- Both positive results target the same problem: one-shot inserts displacing entries with proven reuse. The admission gate (v001) filters them at miss time, and the eviction-next insert position (v003) lets them leave first. Whether these gains overlap or add is not yet known.
- The Winner's gain is recorded only qualitatively. No magnitude is given, so v001 and v003 cannot be ranked against each other on the evidence in their insights.
- Removing the forced demotion (v002) did not recover the loss from the protected tier in Cycle 0004. The protected cap itself, or the second-hit signal, is the more likely cause, consistent with Cycle 0004's suspicion.
- Hit-rate is still the only measured dimension in all three insights. Metadata size and per-op time remain unknown for the shared counter (v001) and the miss-position flag (v003), even though v003's hypothesis claims no new per-entry metadata.
- v002's insight says its overlay could not be read in this run, so its result is not confirmed against the actual diff. The other two insights do not state this.

## Open questions

- Are the admission gate (v001) and the eviction-next insert position (v003) redundant, or do they stack? Testing each alone against the parent would settle this.
- Does the v003 gain hold beyond workload-a, given the Cycle 0004 concern about single-trace results?
- What does the shared counter's metadata and per-op cost amount to once measured?
- Is the protected-cap failure in v002 a cap-size problem or a second-hit-signal problem?

## Continuation

Next cycle should pursue one-shot-insert filtering, testing the admission gate (v001) and the eviction-next insert position (v003) separately or in combination, and confirm the gain on a second workload. Include metadata size and per-op time in the fitness vector before accepting either change. Drop the protected half-bucket with demote-skip as a hit-rate lever, and do not retry it without a changed cap or second-hit signal.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
