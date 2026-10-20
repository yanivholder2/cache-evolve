# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Doubling `ghost_len` to `max_size` did not catch any useful longer reuse distances; hit rate was flat to very slightly down. The extra ghost hits went to keys whose reuse came after a full bucket's worth of evictions. Sending them to probation MRU, instead of the cold `len0 / 2` slot, pushes a fresher probation item toward eviction while gaining nothing. Twice as many live 16-bit fingerprints also means twice as many chances for a false-positive ghost hit.
- per-op-time (put): `take_ghost` runs a linear `position` scan over the whole ring on every non-resident put, and on a cold key that scan finds nothing and so covers every slot. Doubling the ring doubles that scan. Saving the second hash (`fingerprint_from_hash(h)` in `AssociativeCache::put`) did not offset it. `record_ghost` still hashes the evicted key, so only one of the two extra hashes went away.
- per-op-time (get): The get path (`pair_get`/`hit`) did not change at the source level. The regression most likely comes from the larger per-bucket `ghosts` heap allocations, which spread the working set over more memory and add cache pressure on the bucket scans. It could also be measurement variance. Either way, nothing in this change helps get.

## Takeaway

On this LIRS trace, a ghost window of `max_size / 2` per bucket already captures the reuse that ghost-guided placement can exploit. Extending the window to a full bucket adds no hit rate. It also makes every miss-path put pay for a longer linear fingerprint scan. Ghost-ring length is not a productive lever here, so the 3/4 `protected_cap` split or the placement rules themselves are better targets. Single-hash `put` is a fine change to keep, but it is too small to hide an O(ring) scan cost.
