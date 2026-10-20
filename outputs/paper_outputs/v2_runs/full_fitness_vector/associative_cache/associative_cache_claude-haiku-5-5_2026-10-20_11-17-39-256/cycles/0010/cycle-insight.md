# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes: a rotating start cursor for the equal-minimum victim scan (v001), halving the slots per bucket while doubling the bucket count (v002), and lowering the saturation ceiling of the access counter to 3 (v003). Only v002 shows a clear, direction-consistent gain, on the put path. Both listed Winners carry weaknesses: v001 lost hit-rate, and v002's get-side claim is not supported by its median.

## Patterns

- Narrower buckets is the first put-side change in several cycles with a clear effect. Halving the victim scan width lowered both put median and put tail with hit-rate flat. This sits uneasily with Cycles 0007–0009, where shortcuts to the scan and aging did not move the put tail. The likely reading is that the put cost scales with scan width itself, and earlier changes did not reduce that width.
- Tie-break order among slots with equal minimum counters does not protect hit-rate. Rotating the start of the scan (v001) lowered hit-rate, so index-order bias is not the cost. The exact-minimum rule is what matters, and the order in which equal slots are visited is not a lever.
- The frequency ceiling can be lowered without losing hit-rate (v003), so the victim search does not need the full counter range at this size. Both the get and put paths got slower, though, and the insight cannot attribute that to a mechanism.
- Get-tail movement again appears without a get-path change (v001) or with a change that did not target get (v002, v003). The narrow-bucket get-tail tightening in v002 is the one get-side effect here with a plausible mechanism, and it is not confirmed by the median.
- Hit-rate did not move favorably in any variant this cycle. Shrinking buckets did not cost hit-rate on this trace, which is a useful negative result for the associativity trade-off.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the put-side gain from narrower buckets hold across repeated runs, and how far can width drop before hit-rate pays?
- Why did shortcuts to the scan fail in earlier cycles while a narrower scan worked here? Is the difference the width itself or the mechanism?
- What drives the slower get and put paths under the ceiling-3 counter, and does it persist when the ceiling is kept but the counter update is unchanged?
- Is the get-tail tightening from narrower buckets real, given the flat get median?

## Continuation

Next cycle should pursue reducing victim scan width as the put-side timing lever, re-measured over three runs with hit-rate checked alongside, and should not chase get-side claims from it until the get median moves. Drop the rotating equal-minimum cursor (v001) and stop treating tie-break order as a hit-rate lever. Do not keep the ceiling-3 counter (v003) on its Winner label without repeated timing runs, since both of its timing paths regressed.
</content>
