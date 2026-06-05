## Recurring wins

- **Topology changes over policy changes**: expanding hot-tier (normal) bucket capacity yielded the largest hit-rate gain in cycle 1 without modifying any policy logic — structural allocation appears to be the highest-leverage axis for this workload.
- **No regressions from parallel probes**: all three cycle-1 variants improved hit-rate independently, suggesting the baseline has slack in multiple orthogonal dimensions simultaneously.

## Recurring failures

- *(Only one cycle completed — no cross-cycle failure patterns established yet.)*

## Open questions

- Does combining MRU re-entry (v001) with the 50/50 topology split (v003) produce additive gains, or does the expanded hot tier make re-entry position irrelevant?
- Where does the normal/stash ratio gain curve peak or break? The 50/50 split won, but 20/12 or 24/8 splits have not been probed.
- Is LFU's advantage specific to frequency-skewed workloads (workload-a)? It may regress on uniform or recency-heavy traces and should be tested there before being treated as a general improvement.

## Per-policy notes

- **LRU (baseline)**: recency bias hurts stable hot sets under frequency-skewed access; loses items from a hot working set that cycle through stash before earning re-promotion.
- **LFU (v002)**: outperforms LRU when the access distribution is frequency-skewed; benefit is workload-contingent and should not be assumed to generalise.
- **MRU re-entry (v001)**: shortens the time items spend below promotion eligibility after a stash hit; orthogonal to bucket topology and likely additive with v003's space expansion.
- **Topology rebalance (v003 / Dash normal/stash)**: confirmed that the baseline over-provisions stash relative to normal for this workload; equal allocation was strictly better and represents the confirmed winning direction to push further.
