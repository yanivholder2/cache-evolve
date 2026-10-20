# Variant v003 Insight

## Helped

- hit-rate: Scanning up to 4 LRU-end protected candidates and demoting the minimum-frequency one kept slightly more high-value items resident in the protected segment, producing a small but positive gain over the pure-LRU-head demotion in the parent.

## Hurt

## Takeaway

Frequency-ordered demotion from the protected segment does work under the accurate 8× sketch, but the gain is marginal: the PROMO_THRESHOLD=8 gate at promotion time still ensures every protected resident cleared a minimum frequency bar on entry, so genuine frequency spread among the LRU tail of the protected segment is narrow even when estimates are sharp. The k=4 scan window is also tiny relative to the ~1 843-slot protected segment, meaning demotion decisions touch only a small fraction of resident items per eviction cycle. The mechanism is sound and directionally correct, but the binding constraint on hit-rate lies elsewhere — segment sizing or the promotion threshold are likely higher-leverage levers than the demotion scan.
