# Variant v002 Insight

## Helped

- hit-rate: web_0 on 128-wide buckets is the only cell that improved, and only a little. On a small bucket a recently evicted key can come back within the ghost window, and this is the one case where the change actually caught real reuse instead of firing BIP's blind every-32nd-miss promotion.

## Hurt

- hit-rate: hm_0 fell at both bucket widths and web_0 fell clearly on 1024-wide buckets, so the summary dropped below the parent's. The filter has about one bit per recorded eviction (2·W bits, reset after 2·W evictions), so it is not sparse. Late in each epoch most bits are set, and on average roughly a third of all misses test "reused" by false positive. That sends far more one-touch keys to the MRU end than BIP's 1-in-32 did, which removes scan resistance most on wide buckets, where a dead MRU item lingers longest. The hypothesis called false positives "harmless," and that turned out to be wrong. The all-at-once reset also leaves the filter nearly empty just after each clear, so hm_0's shifting working set loses the steady trickle of MRU admissions that BIP guaranteed.
- per-op-time: Put cost rose sharply. Each miss now runs SipHash (`DefaultHasher`) twice, once to test the incoming key and once to record the victim, and every 2·W evictions it also zeroes the whole bit set. Get rose too, even though the get path is untouched. The most likely cause is the larger bucket struct (an extra `Vec` header, mask and counter) plus one more heap allocation per bucket, which spreads bucket headers over more cache lines during lookup. Run-to-run noise may also play a part.

## Takeaway

A hashed ghost bit set only works as a reuse signal if it is sparse. With about one bit per tracked eviction plus a periodic full reset, it fills up and acts like a high-rate random MRU promoter, which is worse than BIP's fixed 1/32 on scan-heavy traces and less steady on shifting ones. A retry would need several bits per tracked key (or fewer tracked evictions), gradual aging instead of a full clear, and a cheap integer mix instead of `DefaultHasher`. Otherwise the per-miss hashing alone wipes out any latency budget.
