# Variant v001 Insight

## Helped

- hit-rate: Recording each miss once (get-miss records, the following put skips via `last_miss`) gives the 4-bit counters finer resolution and delays saturation, with a margin-0 admission rule. The gain over the parent is marginal, within noise of the double-counting setup.

## Hurt

- hit-rate: Nothing measurable regressed. The extra `last_miss` state and hash compare in `put` add a little per-op work and a coupling between get and put, for almost no hit-rate return.

## Takeaway

Halving the sketch inflation and dropping the admission margin to 0 landed almost exactly where the double-counted, wider-margin parent was. The counting scale and the margin compensate for each other, so the frequency gate's behavior is set by their ratio, not by the absolute count. Further hit-rate gains must come from changing what the sketch ranks or how segments are sized, not from re-scaling counts.
