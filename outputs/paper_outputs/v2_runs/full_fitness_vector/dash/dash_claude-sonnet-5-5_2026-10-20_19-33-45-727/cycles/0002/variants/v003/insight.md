# Variant v003 Insight

## Helped

- per-op-time: Both get and put latencies, including the p99 tails, came down against the parent. A plausible cause is that demoted items now go to the LRU end, so cascaded demotions shift less and eviction work is cheaper. I could not read the overlay source, so this reason is inferred from the hypothesis and not checked against the code.

## Hurt

- hit-rate: Hit-rate came out slightly below the parent, so the demote-to-Front rule and the `hit` reset did not stack on the second-hit filter. A demoted item that was still useful now has to earn two more hits and is first in line for eviction. That removes some of the reuse the filter had kept.

## Takeaway

The second-hit admission filter and cold placement for demotions overlap and do not add up. Both try to keep weakly reused items out of the hot region, and the extra demotion penalty cost a little hit-rate while only making operations cheaper. `metadata-size` did not move because the `Item` layout was left alone. The 48-byte item cost from the parent's `hit: bool` is still there, and this variant did nothing to shrink it.
