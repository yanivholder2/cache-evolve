# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Halving every shared `freq` counter each 10× capacity accesses, then dropping entries that reach 0, lowers the incoming-versus-victim counts the admission gate compares, so keys that were admitted on earlier history are now bypassed or evicted sooner than under v001's ever-growing counters.

## Takeaway

Periodic halving with zero-pruning is not a neutral freshness fix on this trace: because `retain(|_, v| *v > 0)` runs right after the halving, every key seen only once in the window loses its history at each reset, which is the same singleton-pruning effect the hypothesis said it avoided. Any future aging for this gate should keep one-hit counts (for example, by halving without removing entries, or by using a longer window) and test that choice separately from the admission rule.
