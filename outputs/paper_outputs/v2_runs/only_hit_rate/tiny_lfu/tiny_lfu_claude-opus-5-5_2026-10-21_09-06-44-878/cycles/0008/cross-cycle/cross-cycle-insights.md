## Recurring wins

- Estimate quality was the only large hit-rate gain in eight cycles. Widening the count-min sketch (cycle 0003) removed collision overcount. Cheap or hash-only changes (fast single hash, cached-hash fast path) kept hit-rate and promised lower cost.
- Making counts honest helped a little. Deduplicating the get-miss-then-put double increment (cycle 0004) let true access counts decide the contest.
- Lifetime, unaged frequency is the asset on workload-a. Keeping strict `>` with keep-victim-on-tie, and protecting long-reuse-distance keys, has held up in every test against it.
- Policy changes now win only by holding hit-rate, or by losing the least. From cycle 0005 on, Winners mostly tied the parent (SLRU split, frequency-based demotion, ghost tie-break, ghost margin).

## Recurring failures

- Aging or decay, tested five ways: aggressive halving, long-period halving, halving on a clean sketch, global halving on SLRU+dedup, and per-entry idle discount. Every variant lost or stayed flat, and later clean tests (cycles 0004, 0008) ruled out the earlier confounders.
- Contest tweaks: tie-admission, ghost tie-breaks, ghost-hit margins and random admission jitter. Exact ties at warm estimates are rare. When the contest is loosened, cold probation keys just churn, and higher-count keys get swapped for lower-count ones.
- Victim sampling depth. Wider or deeper samples evict young, still-useful keys, because without aging the lowest estimate marks the newest resident.
- Resident-structure levers: a 1% W-TinyLFU window (tested on both noisy and clean estimates), a hill-climbing adaptive window, SLRU ratio sweeps, and idle-based protected demotion. The min-estimate victim sample already acts as a soft protected segment, so these rearrange entries without changing which keys get evicted.
- More sketch-accuracy work after the wide sketch: conservative update, more width, more aging. Width and conservative update fix the same error, so their gains don't add up.
- Bundling two mechanisms in one variant (cycles 0001, 0008) made the result hard to attribute.

## Open questions

- Per-op-time and metadata-size have not been scored in any of the eight cycles. Which lineage wins once they are? Candidates are the fast hash, the cached-hash fast path with ≤1 shortcuts, the narrow u16 conservative-update sketch, the SLRU per-hit cost, and the ghost-ring overhead.
- How narrow can the sketch get? Width 8× was neutral without SLRU but slightly negative with it. Would a recency tie-break between equal estimates let it go narrower?
- Two questions remain only to close threads: very long-period halving, and raw-estimate protected-head demotion without a discount. Both are expected to come out neutral or negative.
- Does the workload-a plateau carry over to other workloads, where recency, a window or aging might pay off?

## Per-policy notes

- TinyLFU sketch: a wide sketch plus dedup is the baseline that matters. It is over-provisioned for hit-rate. Counters stay well below u16 saturation.
- TinyLFU admission: keep strict `>` with keep-victim-on-tie. A first-time newcomer that ties a resident accessed once still loses, but this is not where the remaining misses are.
- SLRU (probation/protected) on top of TinyLFU: a marginal win that is already over-defended. Stale-high keys sit in protected, where the contest never reaches them. Demotion policy, ratio and windows did not change outcomes.
- W-TinyLFU window: negative on workload-a (an LIRS-style trace) at fixed and adaptive sizes. Few keys come back within a short recency trial.
- Fast-path coupling: the ≤1 early-stop and early-reject shortcuts are correct only without aging. Any decay or reset proposal must remove them.
- Workload-a hit-rate has been flat or down for four cycles in a row (0005–0008). Treat it as the ceiling for the TinyLFU+SLRU family.
