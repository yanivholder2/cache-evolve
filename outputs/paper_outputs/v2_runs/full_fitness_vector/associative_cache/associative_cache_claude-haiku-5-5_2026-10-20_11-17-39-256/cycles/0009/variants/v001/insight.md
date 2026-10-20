# Variant v001 Insight

## Helped

- per-op-time: The get-hit path now skips the counter store once the counter is saturated, and the get tail (p99) came down slightly, which is the effect the hypothesis predicted for fewer stores on hot hits.
- per-op-time: The put median moved down by a few nanoseconds even though the put path was left untouched, so this is more likely run-to-run spread than an effect of the store skip.

## Hurt

- per-op-time: The get median (the summary metric) ticked up by about a nanosecond, so the skipped store did not improve the typical get and the variant does not beat its parent on the headline number.

## Takeaway

Skipping the saturated-counter write-back trims the get tail but does not lower the get median, so the two timing levers do not stack the way the hypothesis hoped. Hit-rate and metadata size stayed exactly flat, as the change intended, since it only alters when a store happens. The put-side movement is not explained by this edit, and the hypothesis asked for three same-session runs that this single eval does not provide, so the sub-nanosecond differences should be treated as noise until repeated.
