# Variant v002 Insight

## Helped

- hit-rate: The policy stayed the same, as H02 required: 5% small queue, `MAX_FREQ = 7`, graduation at freq > 1, front-of-main re-admission at freq 0, main victims ghosted. The test `matches_ring_reference_on_pseudo_random_streams` confirms it: small, main, ring layout and `ghost_head` all match the reference exactly. So the only behavioural change is the ghost window. In v003 a live-count cap was enforced by the drain loop in `ghost_insert`. Now the bound is ring *position*: the last `main_size` inserts in `ghost_ring`. When a ghost hit removes an entry, it zeroes the slot instead of freeing room, so the next `main_size` inserts can still see keys older than v003's live cap would have kept. The slightly higher hit-rate matches that small widening: a few more small-queue rejects and main victims return through `ghost_map.remove(&fp)` → `main.push_front` instead of starting over in the 102-slot small queue. The gain stays inside the ±0.001 band H02 predicted.

## Hurt

(No measured dimension regressed. The eval only measured hit-rate, so H02's real targets went unmeasured: per-op-time and metadata-size. The code changes those targets depend on did land. `ghost_insert` takes `&K` and stores a `u64` fingerprint instead of cloning `K`. Each ghost entry now costs a `u64` in the ring plus a `u64`→`u32` map entry. Before, it cost `K`+`u64` in the deque plus `K`+`u64` in the map, at up to about 2× `main_size`. The `retain` compaction branch is gone. One new cost is unmeasured: every put that misses residency now hashes the key with `DefaultHasher` (SipHash) to probe `ghost_map`, and the `HashMap<u64, u32>` hashes that fingerprint again.)

## Takeaway

The ghost can be swapped from cloned keys to a fixed-size fingerprint ring without hurting hit-rate. The hole-leaving ring window is a free, slightly wider version of v003's live-count cap. Whether H02 succeeded is still unknown, because its claimed wins were in per-op-time and metadata-size, and this eval did not measure them. This variant needs a cost benchmark before it can be judged against v003. The double hashing per miss (SipHash fingerprint, then a `HashMap` hash over it) is the most likely part to eat into the per-op savings. An identity hasher on `ghost_map` would remove the second hash.
