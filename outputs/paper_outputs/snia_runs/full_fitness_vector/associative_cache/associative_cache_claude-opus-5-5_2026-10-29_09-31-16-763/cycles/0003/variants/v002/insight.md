# Variant v002 Insight

## Helped

- hit-rate: web_0 on 1024-wide buckets got back part of the scan resistance BIP had cost it. One-hit scan keys almost never come back inside the 16-eviction ghost window, so they now always land at quarter depth. No dead item is blindly promoted to the MRU end any more, where it would sit for the whole length of a long bucket. web_0 on 128-wide buckets barely moved.

## Hurt

- hit-rate: hm_0 on 128-wide buckets gave back most of the gain v001 got from BIP. hm_0 on 1024-wide buckets stayed flat, so the overall summary slipped below the parent's. A ghost ring holds only the last 16 evicted keys. A key gets MRU admission only if it returns within those 16 evictions, which is a very short reuse distance. hm_0's shifting working set mostly comes back later than that. As a result, ghost hits are rare, and the bucket acts almost like pure quarter-depth LIP. That is the setup that failed hm_0 on small buckets before BIP.
- per-op-time: Put latency rose again at p50 and at p99. The p99 rise was larger. Every put miss now does a linear scan of 16 `Option<K>` slots for the ghost lookup. After an eviction it also writes a ring slot and updates the index with a modulo. All of this sits on top of the `Vec::insert(len/4, ..)` memmove that still dominates. Get is unchanged, as expected, because the ghost is never touched on the hit path.
- metadata-size: The measured dimension did not move, because it reports `Item` and the top-level cache handle. But each bucket now carries an inline 16-slot ghost array, which is roughly 256 bytes for `u64` keys. This is real per-bucket memory that the current metric does not see.

## Takeaway

Ghost-hit admission keeps scan resistance on large buckets, but a fixed 16-entry ghost is too short to catch hm_0's reuse. On 128-wide buckets, the signal almost never fires, so the hm_0 gain BIP had bought is lost. That gain needs a ghost window sized to the bucket (for example, a fraction of `max_size`) or a cheaper reuse filter, such as a small hashed bit set. Either would see further back for less memory and lookup cost than a linearly scanned key ring, which is now another cost on every put miss.
