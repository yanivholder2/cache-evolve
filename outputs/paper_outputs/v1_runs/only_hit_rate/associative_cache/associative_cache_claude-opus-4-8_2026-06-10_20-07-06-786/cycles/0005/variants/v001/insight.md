# Variant v001 Insight

## Helped

- (none): The intended win — evicting the coldest protected key under full-protection pressure — did not materialize on the measured workload; no dimension improved over the parent.

## Hurt

- hit-rate: Replacing the parent's protected-LRU fallback (`Some(0)`) with a least-frequently-used scan regressed hit-rate. Because every freshly promoted key sits at `lfu_counter == 1`, the min-count scan repeatedly targets recently-promoted keys near the MRU back — and with ties broken toward the LRU end, the saturating tally rarely separates genuinely hot keys from rising ones, so the cache discards the same rising keys v002 worked to keep.

## Takeaway

Under full-protection pressure on this workload, recency (the protected LRU end) is a stronger eviction signal than the banked `lfu_counter` frequency: counting every hit instead of pinning at 1 produces a tally dominated by `== 1` newcomers, so an LFU-with-LRU-tiebreak fallback tends to kill recently-promoted rising keys rather than truly cold ones — the opposite of what the hypothesis intended, and a net loss against the parent's plain LRU fallback.
