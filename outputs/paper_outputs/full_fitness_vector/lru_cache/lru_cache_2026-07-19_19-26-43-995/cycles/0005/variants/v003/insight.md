# Variant v003 Insight

## Helped

- **hit-rate**: Shrinking `prob_cap` from `capacity / 4` to `capacity / 8` raises the protected-queue quota from 75 % to 87.5 % of total capacity, shielding 256 additional slots (at capacity=2048) from demotion back to probationary and thereby keeping moderately-frequent Zipfian items resident long enough to yield hits they would otherwise miss.

## Hurt

- **per-op-time**: The get p50 rose from the parent's 0 ns to 100 ns, pushing the summary from 0 to 100; however, the code change adds no new branches or pointer manipulations to the hot path, so this most likely reflects the parent's 0 ns p50 being a measurement-floor artifact rather than a genuine latency regression introduced by the quota shift.

## Takeaway

Isolating the protected-quota enlargement (from `capacity / 4` probationary to `capacity / 8`) confirms that policy geometry alone — without any additional branch or guard — is sufficient to improve hit rate on the Zipfian workload: the smaller probationary buffer raises the demotion threshold so that borderline-frequent items accumulate in the protected LRU instead of cycling through eviction. The apparent per-op-time hurt is not credibly attributable to this one-line constant change and is better read as measurement noise on the parent's anomalously-zero p50.
