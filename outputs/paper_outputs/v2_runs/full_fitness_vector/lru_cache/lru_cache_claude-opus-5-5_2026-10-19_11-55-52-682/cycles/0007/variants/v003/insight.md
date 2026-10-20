# Variant v003 Insight

## Helped

- hit-rate: Halving the direct-mapped ghost means slots get overwritten sooner, so only keys evicted from the small queue fairly recently still match on return and skip probation into main. Keys that come back after a long gap now go through the small FIFO again and have to prove reuse, which leaves main for keys that were evicted recently. The gain is very small and comes from a single trace at one size. It points the way the hypothesis predicted but is about as large as run-to-run noise.
- per-op-time: Only `put` p50 moved, and only slightly. That fits the probe in `put` (`self.ghost[gslot]`) and the write in `evict` now hitting a smaller table, which is more likely to stay in cache. At capacity 2048 the table drops from 8 KiB to 4 KiB. `put` p99 did not change, so the tail is still driven by SIEVE hand walks and promotions, not by ghost access. `get` never touches the ghost, so its flat numbers are expected.

## Hurt

- metadata-size: Did not move, as expected. `Node` layout and the `LruCache` struct header are untouched, and the summary does not measure the ghost's heap buffer. In fact the change halves that buffer, a real but unmeasured gain.

## Takeaway

Making the ghost shorter at capacity 2048 (1024 slots, still about 5x the roughly 204-entry small queue) did not lose any hit-rate, so the hypothesis's falsification condition was not met. The small rise supports, without clearly showing, the claim that bounded ghost reach acts as a useful admission filter. The ghost can be halved with no hit-rate or latency cost and with half the heap. Its size now looks like a weak lever on workload-a, so another halving would mainly tell us where hit-rate starts to fall, not bring a meaningful gain.
