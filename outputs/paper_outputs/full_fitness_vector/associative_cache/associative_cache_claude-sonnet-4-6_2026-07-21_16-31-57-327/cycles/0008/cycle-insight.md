# Cycle 0008 Insight

## Cohort outcome

All three variants probed the second-chance scan's internal parameters — window depth and scan direction — rather than its interaction with promotion. The two winners confirmed that the 8-slot window is already near-optimal in depth, and that reversing iteration order to evict the oldest cold candidate first is a free hit-rate gain.

## Patterns

- **Scan direction is the highest-value tunable found in the second-chance scan.** Switching from newest-cold-first (LIFO-among-cold, tail position) to oldest-cold-first (LRU-among-cold, inner edge of window) improved hit rate at zero latency cost. The parent's LIFO ordering was silently discarding items too young to have received a second request; LRU-among-cold preserves them long enough to convert.
- **8-slot scan depth is already the sweet spot; neither direction of change helps.** Halving to 4 positions degraded p99 latency without any hit-rate movement — on the rare evictions where cold items don't occupy the bottom 4 positions, the narrower window falls back to a worse candidate and spikes tail latency. Expanding to full bucket depth added unbounded comparison work with zero hit-rate return, confirming that never-accessed items beyond the 8-slot tail are essentially absent under the 3-swap promotion regime.
- **Scan cost is positional, not directional.** The iteration-order flip in v003 carries no measurable overhead because scan length is unchanged; all the per-call work budget is in the traversal count, not the comparison order.

## Open questions

- Is the LRU-among-cold ordering optimal, or would a frequency-weighted tie-break within the cold window (e.g., evict the item with the fewest accesses among the cold candidates) add further hit-rate lift on this workload?
- With scan direction confirmed fixed and scan depth confirmed fixed, is there remaining headroom in the second-chance flag's definition — e.g., expiring or resetting the "ever-accessed" bit after N eviction passes to prevent stale warm markers from permanently shielding aging items?

## Continuation

The next cycle should treat the 3-swap loop + 8-slot second-chance scan + LRU-among-cold eviction order as the new confirmed base, replacing the prior LIFO ordering throughout. The remaining structural unknown from Cycle 0007 — what actually caused the put p99 regression in Cycle 0006's v003 — has not been resolved and should be targeted directly: compare the new base against the original 8-slot LIFO configuration on put-heavy access patterns to isolate whether the regression was a metadata-layout artifact, a scan-fallback artifact, or something else. Flag-lifetime and frequency-within-cold experiments are the natural next frontier if the put regression diagnosis comes back clean.
