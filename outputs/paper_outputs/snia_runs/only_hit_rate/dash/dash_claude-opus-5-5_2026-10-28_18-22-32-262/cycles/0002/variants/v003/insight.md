# Variant v003 Insight

## Helped

- hit-rate (`hm_0` at the large size only): At the large size there are many stash buckets, so the gap between a key's first put and its first reuse is short compared with how often its own bucket takes new inserts. A fresh key can therefore still be hit while it sits at the LRU end. Putting `put` arrivals at `Front` then did what H03 intended: one-hit-wonder keys were evicted ahead of demoted, already-reused items, which v001 places at `Back`.

## Hurt

- hit-rate (`web_0` at both sizes, `hm_0` at the small size): Each stash bucket holds only a few slots, so a fresh key inserted at `Front` (index 0) is the next item evicted. One later insert into the same bucket, whether another put or a demotion, removes it. A key whose first reuse comes even a little later loses the single stash hit that `promotion_get` needs to promote it to the target. On these traces that lost reuse cost more than protecting demoted items gained. The H03 rationale named this risk for the small size, and it also appeared on `web_0` at the large size. The bimodal escape did not save these keys. `put_counter` is counted per segment, not per stash bucket, so a given bucket gets a `Back` insert only by chance, about once every 32 puts across the whole segment. That is far too rare to protect keys reused at a moderate distance.

## Takeaway

v001 already fixed the ordering H03 targets: demoted items rank above fresh puts. Moving fresh puts further down to the LRU end of a tiny stash bucket does more than reorder them. It leaves new keys with almost no time in the cache. With bucket-local LRU and only a few slots, `Front` insertion works as "evict on the next collision", so the stash can no longer act as a probation stage where a new key gets one chance at a hit. Pure-LRU BIP from Qureshi et al. assumes a set with many ways. Here, insertion policy for fresh arrivals only helps when buckets are large or reuse comes soon after the first put, as in `hm_0` at the large size. A softer variant would be better: insert one slot above the LRU end, or bias toward `Front` only when the bucket already holds a demoted item. Keep the bimodal counter per stash bucket.
