# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's sampled probation victim (coldest of the 4 oldest) and followed its continuation. They tried single-count sketch accounting stacked on top (v001), widening the victim sample from 4 to 8 (v002), and a +1 ghost re-entry bonus for recently rejected candidates (v003). v002 was named winner, but its own insight puts its summary *below* the parent: small-size gains came with a loss on web_0 at the large size. So the win is relative to the cohort, not progress over v003. No variant reached any of its stated targets.

## Patterns

- web_0 at the large size was the failure point for every variant. Each change made the admission bar easier to clear or removed a newcomer advantage there. The large-size frequency filter is the lineage's most valuable and most fragile asset.
- The mechanisms overlap rather than add. Single counting, a wider minimum, and the sampled victim all push admission toward "favour accumulated frequency, evict whatever looks coldest". Stacking them gives diminishing or negative returns. The coldest-of-4 sample looks close to the sweet spot. A minimum over more noisy count-min estimates picks residents that only *look* cold.
- The unaged sketch is the common root cause. v001 showed that the "inflated" estimate of 2 on miss-then-insert was the only recency weight newcomers had. v003 showed that, without decay, ties happen almost only among cold keys, so a tie-only bonus just reshuffles cold entries. Two variants independently concluded that the lineage needs real aging or decay before recency-side tweaks can work. This reverses the cycle 0003 verdict on halving, which was made before SLRU and victim sampling existed.
- Any horizon that scales with capacity is risky at the large size. A ghost of capacity/2 let scan or loop keys on web_0 return while still remembered. This repeats the earlier lesson about controllers whose timing scales with capacity.
- hm_0 small dropped again, in two of the three variants. Six cycles of hypotheses aimed at it have not moved it.
- Metadata-size and per-op-time are still unmeasured, now six cycles in a row. v002 doubled the sketch estimates per insert and v003 added a hash set, both for no hit-rate gain.

## Open questions

- Now that SLRU and victim sampling are in place, does sketch aging (halving, or a doorkeeper reset) give newcomers back a recency weight? Does it then make single counting and re-entry bonuses work?
- Would a contest margin or a less noisy victim estimate (for example, the median of the sample, or a conservative-update sketch) sharpen admission better than a wider minimum?
- Does a short, fixed ghost horizon with a bonus that matters at non-trivial counts help hm_0 without leaking scans on web_0 at the large size?

## Continuation

Treat cycle 0005 v003 (4-node sample, double counting kept) as the real best base. v002's 8-node sample did not beat it. Stop sweeping the sample size and stop stacking single counting without decay. The next cycle should test sketch aging or decay on this base as its own change, then retry single counting or a re-entry bonus on top of it only if aging holds web_0 at the large size. Any ghost or bonus must use a fixed, short horizon. Measure metadata-size and per-op-time.
