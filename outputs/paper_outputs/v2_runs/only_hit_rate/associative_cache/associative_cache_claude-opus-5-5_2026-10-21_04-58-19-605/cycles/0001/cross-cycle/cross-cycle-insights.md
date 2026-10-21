## Recurring wins

- (Only one cycle so far, so nothing has recurred yet. Strongest early signal:) Insert new keys with low protection and promote a key only on its second hit. Two independent mechanisms both beat LRU hit-rate on workload-a this way: midpoint insertion and CLOCK with the ref bit cleared on insert. Filtering out keys used only once looks more valuable than exact recency order.
- Exact per-bucket LRU ordering doesn't seem to be needed for hit-rate. An approximate CLOCK ring matched or beat it.
- Removing the per-item wall-clock timestamp is neutral for hit-rate. Shrinking the item record looks safe.

## Recurring failures

- Gaps in evaluation: per-op-time and metadata-size were never recorded. Hypotheses about latency or size can't be confirmed until both are collected. Treat any claim about them without a measurement as untested.
- Ordering by wall-clock time adds cost and no hit-rate benefit on the ClassicLRU eval path. Don't pursue it further.

## Open questions

- Is midpoint insertion's work on each miss (shifting about half the bucket) worth more than its hit-rate gain once per-op-time is measured? Does CLOCK's bit sweep on misses cancel out its cheap hits?
- Is the CLOCK hit-rate gain fully explained by insertion position, or does second-chance add something beyond it?
- How deep should new keys be inserted? Can CLOCK with cleared-on-insert bits plus a compact item record win hit-rate, latency and size together?
- Does a shared global atomic logical clock hurt multi-threaded throughput?

## Per-policy notes

- LRU with midpoint insertion: best hit-rate so far. Its per-op-time cost on misses has not been measured.
- CLOCK ring (in-place ref bits): beat LRU on hit-rate as a side effect. The claimed latency win is unmeasured.
- LRU with logical u32 stamps: identical hit-rate to the baseline, as predicted. The size and concurrency effects are unmeasured.
- Baseline ClassicLRU: beaten on hit-rate by both variants that give new keys low protection.
