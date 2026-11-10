# Variant v003 Insight

## Helped

- hit-rate: Essentially flat, which is what the hypothesis expected. hm_0 at 131072 moved up slightly, and the other three trace/size points moved within noise. The ring's window is the last `main_size` ghost insertions, so slots made stale by ghost hits shorten the live history a little. On the large hm_0 cache this may filter out a few marginal long-distance re-references that would otherwise be promoted straight into main. The effect is too small to count as a real policy gain.

## Hurt

- per-op-time: `put` p99 rose clearly, `put` p50 and `get` p50 edged up, and the summary got worse. That is the opposite of what the hypothesis claimed. Once the ring has wrapped, `push_ghost` now does a `take()` on the head slot, a `ghost_index.get` on the evicted key, and usually a `remove`, on every ghost insertion. The parent did that eviction probe only when the live count was at `main_size`, and it amortized stale tuples. The parent's rare O(2·main_size) `retain` and `pop_back` scans were evidently not what set p99. Constant extra hash probes per miss cost more than those rare spikes did. The memory saving is also smaller than claimed. `Option<(u64,u64)>` takes 24 bytes per slot, not 16, so the preallocated `main_size` ring saves only about a quarter of the parent's `2·main_size+1` deque reservation, not half. All `main_size` slots are also written in a strided pattern from the start.

(metadata-size did not move. `Entry<u64,u64>` stays 24 bytes, and `S3Fifo<u64,u64>` stays 232 bytes because the `VecDeque` field was replaced by a `Vec` plus a `ghost_head: usize`.)

## Takeaway

Making ghost maintenance strictly O(1) per insertion did not lower the `put` tail. The parent's compaction and scan loops were amortized rarely enough that they stayed off p99. The ring instead adds a cold hash probe for the evicted key on every ghost insertion, plus wider `Option`-wrapped slots. The `put` tail is driven by the number of `ghost_index` probes per miss, not by occasional O(n) cleanup. Future tail work should cut hash probes on the miss path, for example by not tracking eviction liveness with a separate lookup. Swapping the ghost container shape is not the lever.
