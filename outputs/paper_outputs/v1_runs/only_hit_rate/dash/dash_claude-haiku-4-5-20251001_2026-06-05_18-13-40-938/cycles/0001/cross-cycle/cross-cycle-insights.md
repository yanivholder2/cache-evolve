# Cross-Cycle Insights

## Recurring wins

- **Stash-hit acceleration pays off**: Both winning strategies (v001, v003) prioritized fast-tracking stash residents, suggesting this is a robust lever for improvement. The workload clearly rewards treating stash occupancy as a strong signal of interest.
- **Position tuning within tiers matters**: v001's shift from front (cold) to back (MRU) insertion in the same tier yielded gains, indicating that traversal order and promotion eligibility are sensitive to placement within a tier.

## Recurring failures

- **Frequency-based eviction misfires on this workload**: v002's LFU policy showed no improvement despite correct implementation. Recency dominates the access pattern; access count is a weak signal compared to temporal locality.

## Open questions

- **v001 vs. v003 trade-off**: Both improved hit rate, but v001 was selected. Does v003's tier-skipping cost outweigh its placement priority on tail latency or other subsystems?
- **Synergy potential**: Can v001's within-tier positioning and v003's tier bypass be combined, or do they interfere?
- **Optimal acceleration strategy**: Is there a middle ground between tier-by-tier filtering and full bypass for stash hits?

## Per-policy notes

- **Stash-tier interactions (v001, v003)**: The two wins both centered on stash residents but diverged on how to promote them—either by repositioning within a tier or by skipping tiers. Both approaches beat the baseline, pointing toward acceleration as the unifying principle.
- **Access-frequency policies (v002)**: LFU's failure reinforces that this workload's benefit from recency-based signals far exceeds what frequency can offer. Future frequency variants should be deprioritized unless the traffic pattern shifts.
