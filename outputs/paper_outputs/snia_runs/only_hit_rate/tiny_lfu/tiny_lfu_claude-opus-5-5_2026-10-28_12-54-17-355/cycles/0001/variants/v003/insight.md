# Variant v003 Insight

## Helped

- hit-rate: Halving every sketch counter every 10×capacity increments (plus the 15 cap) stops keys that were hot early from holding onto very large counts. Newly popular keys can now beat the LRU victim in the admission check. The biggest gains are at the small (16384) size on both web_0 and hm_0, where the old resident set stayed stuck the longest and each admission decision matters most.

## Hurt

- hit-rate (web_0 @ 131072 only): This one cell dropped slightly; the other three improved. At this size the aging period is very long (about 1.3M increments), so aging adds little there. The 15 cap, however, makes many hot residents tie at the ceiling. Because `put` requires `incoming_freq > victim_freq`, tied newcomers are rejected, and the sketch can no longer rank hot keys against each other. This is the likely cause, though the eval does not separate the effect of the cap from the effect of aging.

## Takeaway

Aging the sketch is a clear net win for this TinyLFU over the MSR traces. Old frequency history was blocking admission, and the effect is strongest when the cache is small compared with the working set. The hypothesis also added a 4-bit-style cap, which is mixed in with the aging change. The one regression, at the large size, points to the cap squashing frequency differences rather than to aging. A next step is to test aging without the cap (or with a higher one) and to scale the aging period by something other than capacity at large sizes. The metadata-size and per-op-time dimensions are missing from this eval, so the claim that "storage/latency is unchanged" is still unverified.
