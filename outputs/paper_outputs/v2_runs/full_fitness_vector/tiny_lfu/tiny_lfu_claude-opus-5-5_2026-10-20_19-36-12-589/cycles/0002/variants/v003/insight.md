# Variant v003 Insight

## Helped

- hit-rate: Choosing the coldest of the 4 LRU-end entries as the victim, instead of always the LRU head, gave only a marginal gain that is within run-to-run noise. On workload-a, the sampled entries near the LRU end usually have similar sketch estimates. The tie-break goes to the older node, so the change mostly reduces to the parent's head-eviction. It also lowers the bar for admission only when a strictly colder entry sits just behind the head, and that is rare.

## Hurt

- per-op-time (put): Every full-cache miss now runs up to 4 `key_hash` calls plus 4 four-row sketch estimates for the candidates, where the parent ran one of each. This undid a large part of the put-tail improvement that the parent's single-hash design had gained. Put p50 also went up. Get is unchanged because only the full-cache branch of `put` was touched, which also leaves the headline per-op-time summary flat.

## Takeaway

On this trace, using the sketch to pick the victim from a small LRU-end sample adds almost no hit rate: the sampled candidates are near-equal in frequency, so the admission test still decides almost everything. It also brings back the per-miss hashing cost the parent had just removed. If cycle 0001 was right that the admission signal caps hit rate, the cap is not caused by which LRU-end victim is picked. A sample of 4 is too small or too uniform to change it, so this design gives up put latency for no real gain.
