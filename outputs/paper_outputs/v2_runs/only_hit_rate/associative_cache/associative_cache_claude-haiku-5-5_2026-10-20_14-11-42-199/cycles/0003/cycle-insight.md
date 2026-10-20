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
