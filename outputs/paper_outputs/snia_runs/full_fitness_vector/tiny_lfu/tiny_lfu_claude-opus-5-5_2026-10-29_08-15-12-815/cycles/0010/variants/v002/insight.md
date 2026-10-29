# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): This is the only cell that rose, and only by a marginal amount. It fits the mechanism working occasionally: a warm probation head escapes eviction when its successor is colder. The gain is too small to separate from noise, and it is at the small size, where the hypothesis predicted no change.

## Hurt

- hit-rate (web_0 @ 131072): The targeted cell went slightly down, not up, so the hypothesis is refuted on its own success criterion. Sampling the successor `b` evicts a younger probation entry before its first re-reference can promote it. Meanwhile the older head keeps its place on frequency that is aging out of the sketch. This is the "fresh-but-cold" failure mode the rationale itself named.
- hit-rate (web_0 @ 16384): This cell had the largest loss. Choosing the colder of two victims lowers the frequency bar a window candidate must beat in `admit()`. That loosens the TinyLFU filter: with the parent's tie rule (cand ≥ 2 wins ties) still in place, more marginal candidates get into probation. On a small, recency-driven web_0 cache, those admissions push out entries that would have been re-hit.
- hit-rate (hm_0 @ 131072): A small loss from the same effect. In a large probation segment, the head's successor is nearly as old as the head, so preferring it by a strictly lower estimate mostly picks between sketch-noise-level differences. It gains no real coldness signal and still pays the admission-bar cost.
- hit-rate (summary): The losses in three cells outweigh the marginal hm_0 @ 16384 gain.
- per-op-time (put p50/p99): `evict_one` now does a second `key_hash` and a second `estimate_hashed`. It also does a dependent load of `node(a).next` followed by a read of `b`'s node, which is a random access into the node array and likely a cache miss at large capacities. All of this sits on every full-cache insert, so a slower put matches the change.
- per-op-time (get p50/p99): `get` and `on_hit` are byte-identical to the parent, yet the get tail grew sharply. The diff cannot cause this. Like the earlier swings the parent's insight attributed to benchmark noise or layout effects, it should be read as measurement variance or a code-layout change from the larger `evict_one`, not as a real hot-path cost.

## Takeaway

Sampling a second probation victim works on the victim side of the contest, but in practice it does the same thing as a candidate-side bonus: it lowers the bar for admission. On top of v002's tie-win rule, the loosening stacks and hurts web_0 most. Under LRU-ordered probation, "colder by estimate" mostly means "younger and not yet re-referenced", so evicting the successor punishes recency instead of finding stale entries. A useful victim-side refinement would need an age-aware or stale-frequency signal, not a raw estimate comparison against the immediate successor.
