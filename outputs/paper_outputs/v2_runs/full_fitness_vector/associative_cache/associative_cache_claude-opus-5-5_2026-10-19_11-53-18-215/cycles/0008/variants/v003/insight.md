# Variant v003 Insight

## Helped

- hit-rate: Cold keys (ghost misses) now take the evicted victim's slot at probation LRU instead of being rotated to `len0 / 2`. A one-hit wonder is therefore the next victim. The protected item that was just demoted to probation MRU, and keys re-admitted on a ghost hit, now survive at least one more cold miss before they are evicted. This acts as LIP/BIP-style two-strike admission without shrinking or displacing protected. The gain is modest, which fits what the code shows. When probation has 1 slot, `mid = len0 / 2 = 0`, so v001's rotate was already a no-op and the new code behaves exactly like the old. The behaviour only differs when probation has ≥2 slots: the floor-of-2 buckets of 16+ slots, or a probation that is temporarily wider after demotions.
- per-op-time (put): The full-bucket cold path is now just two `mem::replace` calls plus `record_ghost`. The `rotate_left(1)` over `items[0..=mid]` (16-byte items) and over `tags[0..=mid]` is gone. The drop is larger at the tail than at the median, which fits a saving that only appears when probation is wide enough for `mid > 0`. While the bucket is filling, the extra cost of `insert(0)` compared with `insert(mid)` is too small to matter.

## Hurt

None. metadata-size is unchanged because `Item` and the cache header were not touched. per-op-time (get) is flat, as expected: the lookup and hit paths (`find_tagged`, the protected/promotion rotates) were left alone.

## Takeaway

When probation is only 1–2 slots, where a cold key enters is still a real hit-rate lever. It works because it decides whether the newcomer or the just-demoted protected item gets evicted next. Placing history-less keys at LRU improves both hit rate and put cost at the same time, since it removes work rather than adding it. This is the first hit-rate gain on this lineage that leaves the protected cap alone. Admission policy for cold keys, not further cap tuning, is the axis that still has room. But with a 1-slot probation, LRU entry and middle entry are the same thing, so this lever is limited by how wide probation is.
