# Variant v002 Insight

## Helped

(No dimension improved. hit-rate is the same as the parent in every cell, as the hypothesis required. The trim only pops tuples that are already stale, so which keys are live ghosts and their FIFO order do not change. The new `incremental_trim_preserves_live_ghost_fifo` test checks this against a reference model. metadata-size did not move either, because the change adds a constant and a loop but no fields.)

## Hurt

- per-op-time: The `put` tail got noticeably worse, and `put`/`get` p50 drifted up slightly. The trim loop runs on every `push_ghost`, which means on nearly every miss that evicts from small. Even in the common case, where the back tuple is live, it still does at least one extra `ghost.back()` read and one `ghost_index.get` probe. Both touch the oldest, coldest part of a deque and hash table sized to `2 * main_size`, so the new cost lands on every miss instead of on rare ones.
- per-op-time: The work the trim was meant to remove probably never showed up at p99. The `retain` compaction can only fire after about `main_size` stale tuples pile up, so it runs on far fewer than 1% of puts and sits beyond the p99 percentile. Also, once the ghost queue is full, the existing "pop_back until the oldest live tuple is removed" loop already drops stale tuples that reach the tail. The new trim mostly repeats that cleanup and adds probes without removing much.

## Takeaway

Spreading the ghost `retain` into bounded per-insert tail checks keeps the policy unchanged but is a net loss for latency. It turns a rare O(n) event that p99 cannot see into a probe of cold memory on every miss. In the full-ghost regime, the tail was already being cleaned by the eviction pop loop. Future `put`-tail work should target costs that happen more often than 1% of puts, and should not add any unconditional per-miss lookup, even one that exits early. Cycle 7 made the same mistake: its per-insert liveness probe also always ran.
