# Variant v003 Insight

## Helped

- hit-rate: Items demoted from the normal tier (both the direct `true_target == source_idx` path and the cascade path) now re-enter stash at the MRU end through `DEMOTE_TO_STASH_POS = Back`, not at index 0. Under ClassicLRU they are no longer the next stash victim. The `PROMOTE_THRESHOLD - 1` pre-credit they carry now has time to pay off, and one return hit puts them back in their target. In the parent, a newly demoted item was usually pushed out before that return hit came.

## Hurt

## Takeaway

The parent's demotion pre-credit only worked if retention also changed. Crediting an ex-resident with a stash hit does little when the same code places it at the stash LRU end, where it is evicted next. Moving it to MRU makes the stash behave like a victim cache, similar to SLRU/2Q. It costs nothing in normal-tier moves, because the target one-more-chance retry still uses the incoming `insert_pos` and fresh `put` keys still enter stash at the default position. The gain is modest and was measured on a single LIRS trace at one cache size. Whether MRU placement crowds out one-touch keys that are about to get their second hit is still untested on scan-heavy or larger-working-set workloads.
