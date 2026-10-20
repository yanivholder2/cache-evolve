# Variant v001 Insight

## Helped

- per-op-time (get): The hypothesis targeted the get median, but only the tail improved. The most likely cause is that protected and probation hits now take the same code: one `items[pos..]`/`tags[pos..]` rotate, then a branch-free `len0` update behind a single `fast_slru` check. Before, the parent branched on `pos >= len0` and then on `protected_cap == 0`. On a mixed stream of hit kinds, those branches mispredict now and then, and that cost shows up in the tail, not the median. Removing the per-op `debug!` level check, and the `data.is_some()` evaluation that fed it, cuts some more occasional overhead.
- per-op-time (put): Both the put median and tail moved down. Put gains twice. Its own `debug!` call is gone, and a resident-key update (`pair_put_tagged` → `hit`) now takes the same unified hit path as a get. The new-key path through `insert_new` is unchanged, which explains why the gain is modest.

## Hurt

None. hit-rate is unchanged, as intended: the side-by-side reference test shows `items`, `tags`, `len0` and `meta` match the cycle-0009 hit rule after every op. metadata-size is also unchanged, because `fast_slru` lives in the bucket, and neither `Item` nor the cache header changed.

## Takeaway

Cutting fixed instructions from the hit path did not move the get median. That suggests the median is now set by the tag scan and the rotate itself, not by the per-op checks around them. What branch-free bookkeeping plus dropping hot-path logging did buy is lower tail latency on both get and put, at zero hit-rate cost. That makes this a free tail-latency win. It also suggests that further get-median gains will need a cheaper `find_tagged` or a shorter rotate, not more control-flow trimming.
