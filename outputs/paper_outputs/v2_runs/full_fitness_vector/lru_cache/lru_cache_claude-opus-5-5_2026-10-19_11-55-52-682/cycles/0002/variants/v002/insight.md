# Variant v002 Insight

## Helped

- hit-rate: The 2-bit saturating counter lets the sweep tell keys hit once in the last revolution apart from keys hit repeatedly. Frequently reused entries now survive up to `FREQ_MAX` passes of the hand, while one-touch keys (inserted at `freq = 0`) are evicted on the first pass. Workload-a at size 2048 rewarded this extra frequency signal over the parent's single reference bit.

## Hurt

- per-op-time: The `put` tail grew noticeably and the `put` median grew a little. The extra sweep work is the likely cause: a counter of 3 needs up to three decrement-and-advance passes before its slot can be evicted, and the bound on the worst-case sweep rises from about one revolution to about `FREQ_MAX` revolutions. On this trace, enough of the ring stays non-zero that the longer sweeps appear in `put` p99. This is the risk the parent insight flagged.
- per-op-time: The `get` tail rose slightly while the median stayed flat. The hit path still never relinks. It changed from an unconditional bit store to a read-compare-increment on `freq`, which adds a small data dependency but does not touch any extra memory.

(metadata-size did not move. `freq: u8` reuses the padding byte that `referenced: bool` used, so `Node<u64,u64>` and `LruCache<u64,u64>` keep their parent sizes.)

## Takeaway

Upgrading CLOCK's reference bit to a 2-bit GCLOCK counter costs no memory and improves hit rate. The cost lands in the eviction sweep: on this trace, deeper per-slot credit makes `put` sweeps longer, so `put` p99 pays for the hit-rate gain while `get` stays almost as cheap as before. Two follow-ups could cut sweep length while keeping most of the frequency signal: a smaller `FREQ_MAX`, or decrementing by more than one per pass. Both are worth testing.
