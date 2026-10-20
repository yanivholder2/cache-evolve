# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: bucket repartitioning, an aging LFU policy, and a static-dispatch refactor of the bucket policy. Only v001 (fixed-width buckets) won, and its hit-rate gain was marginal. v002 and v003 showed no hit-rate movement and carried no measurements on other dimensions, so neither is judged yet.

## Patterns

- Hit rate was the only dimension measured in this cycle. Every Winner-or-not decision rests on it, and the gain that earned v001 its place was small.
- The latency and metadata-size effects the hypotheses predicted were not measured in any variant, so they are untested rather than refuted.
- Two of three variants left hit rate unchanged. For v002 this may mean the evaluation never selected the new policy; for v003 the refactor was meant to be neutral on hit rate, so no change is expected.
- Smaller bucket-conflict misses appear to be a minor share of total misses at this cache size, which limits how much further geometry changes can gain.

## Open questions

- Does the evaluation path actually select the aging policy in v002, or is the fixed aging period too coarse to alter eviction order?
- Does the per-operation cost of scanning wider buckets (v001) show up once per-op time is measured?
- Does the static-dispatch change in v003 reduce get latency when measured separately from put?

## Continuation

Next cycle should pursue changes whose effect can be checked on per-op time and metadata size, and include those dimensions in the fitness vector. Re-test the aging policy only after confirming the evaluation selects it. Drop further hit-rate-only geometry tweaks unless they are paired with a cost measurement.


### Cycle 0002

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
