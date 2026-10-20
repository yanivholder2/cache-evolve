## Recurring wins

- Reuse-aware eviction is the direction that has improved hit-rate so far; both evaluated variants gained on the LIRS workload-a trace (only one cycle of evidence so far).
- Keeping the overwrite (put-existing) path free of list surgery looks cheap and worth preserving; the referenced-bit variant got cheaper there.

## Recurring failures

- Hit-rate gains have come with per-operation cost increases. Both policy changes paid for their gains in the hot paths, so hit-rate alone has not been enough to justify a variant.
- Per-node and per-cache metadata growth has accompanied every variant tried so far and has shown up as a cost each time, without a clear hit-rate payoff attributable to the extra fields.
- Dependency or build failures are not evidence against a strategy. The v003 attempt died at dependency fetch, so its design is still untested and should not be counted as a failure.

## Open questions

- Do the hit-rate gains hold on workloads without one-hit scans, where segmentation has less to protect?
- Is the get-path regression in the referenced-bit variant real or measurement noise?
- Can the second-chance eviction scan be bounded so the put-path tail stops worsening?
- Can the segmented variant keep its hit-rate while dropping the extra per-operation pointer work?
- Does the v003 strategy beat the current baselines once its dependency fetch is reachable?

## Per-policy notes

- Segmented (probationary/protected) recency lists: improved hit-rate, but added promotion and demotion work on get and put and extra per-node state. Worth retrying only if the pointer work can be removed.
- Referenced-bit / second-chance eviction: improved hit-rate and made overwrites cheaper, but the eviction scan on misses is variable-length and worsened put-path tail behaviour. Bounding the scan is the open lever.
- Unevaluated variant (v003): no policy evidence yet; treat as an open thread rather than a known result.
