# Cycle 0003 Insight

## Cohort outcome

Two variants tried to shorten the get scan by changing scan order or key placement: v001 reversed the scan from the MRU end under LIP/BIP, and v003 used swap-halfway self-organization under CLOCK. The third (v002) replaced BIP's fixed-period MRU admission with a 16-entry ghost ring. v001 and v002 are the Winners, but both wins are marginal. v001 changed nothing measurable beyond noise. v002 traded the hm_0 small-bucket gain for web_0 large-bucket scan resistance and paid for it in put latency.

## Patterns

- Changing scan order or key placement does not move get p50. Reversing the scan gave about a noise-level gain. Self-organizing swaps made it clearly worse, because they turned CLOCK's one-byte hit store into writes across two cache lines. Across three cycles, the lookup cost tracks how many full-key compares are done, not where the hit sits. Hits under quarter-depth insertion are not concentrated at either end.
- Hit-path writes must stay as cheap as a bit store. Any reordering on hits (v003) gives back CLOCK's latency advantage and also slightly disturbs the hand's eviction order.
- A reuse signal only works if its window matches the workload's reuse distance. A fixed 16-key ghost catches nothing on hm_0's slow-shifting set, so on small buckets it behaves like plain LIP. It does correctly keep scan keys out of MRU on web_0. Neither BIP's blind period nor the short ghost wins on both workloads.
- Every extra structure on the put-miss path again raised put p50 and p99. The `Vec::insert` memmove at quarter depth is still the main cost, with the new work added on top.
- The metadata-size metric is blind to memory held inside each bucket. This is now confirmed twice (key mirror, ghost ring), so do not treat flat metadata as evidence that a change is free.

## Open questions

- Would a ghost window sized to `max_size` (or a hashed bit-set reuse filter) catch hm_0 reuse on 128-wide buckets without losing web_0?
- Does a compact inline tag/fingerprint pre-filter actually cut get p50? All three cycles' insights point to it, but it has not been tested yet.
- Can quarter-depth insertion avoid the memmove, for example with a segmented or CLOCK-like approximation of LIP, so that put latency stops growing with every policy tweak?

## Continuation

Stop changing scan order and hit-time reordering: both are exhausted. Make the next speed hypothesis a compact fingerprint/tag pre-filter that reduces full-key compares while keeping the hit-path write a single store. For hit rate, pursue a reuse filter that scales with bucket size (a sized ghost or a hashed bit set) as a replacement for both BIP's period and the fixed 16-slot ring. Separately, target the cost of the insert-at-depth memmove. Treat flat metadata-size as uninformative whenever a change adds per-bucket state.
