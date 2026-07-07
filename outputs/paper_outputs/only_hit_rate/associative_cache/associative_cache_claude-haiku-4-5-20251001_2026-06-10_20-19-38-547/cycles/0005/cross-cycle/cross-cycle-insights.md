# Cross-Cycle Insights

## Recurring wins

- **Capacity scales faster than selectivity within a mechanism**: Both Cycles 0001–0002 demonstrated that raw stash capacity expansion outperformed more sophisticated insertion filtering or eviction ordering. Expanding the recovery window from 10% to 25% delivered consistent gains, while LRU ordering and occupancy-aware filtering added minimal value.

- **Orthogonal levers unlock productivity after saturation**: Cycles 0002–0003 showed stash-only refinement facing diminishing returns (1.2% gain, then 0%). Cycle 0004 broke through by adding geometric rebalancing, validating that complementary mechanisms (geometry + stash) compound more effectively than deepening a single axis.

- **Unconditional insertion outperforms selective filtering**: Across Cycles 0003 and 0005, occupancy-aware and collision-aware filtering both regressed or showed zero gain. Blanket FIFO insertion at 25% capacity appears to capture all re-use-worthy items; the signal of which evictions are "worth" preserving does not correlate with recovery value.

## Recurring failures

- **Filtering cannot improve over unsaturated capacity**: Introduced in Cycle 0002 (occupancy-aware) and reattempted in Cycles 0003 and 0005 with different signals (collision-awareness), filtering consistently failed to compound gains at the proven 25% capacity. The root cause: a properly-sized stash already captures the entire value-bearing subset of evictions, leaving no low-value pollution to filter out.

- **Stash-internal ordering refinements plateau quickly**: LRU eviction ordering (Cycles 0002, 0004) showed small improvements relative to FIFO but never won in direct comparison. This pattern suggests that once stash capacity is sufficient to hold the working set's re-use window, the order in which items are evicted becomes secondary; the bottleneck has shifted elsewhere.

- **Architectural constraints block systematic capacity exploration**: Cycles 0003 and 0005 encountered identical borrow checker failures when testing capacity points beyond 25% (20%, 30%). This unresolved constraint has prevented mapping the efficiency frontier and saturation boundary, leaving a key design axis unmeasured.

## Open questions

- **Geometric saturation**: Is the observed 0.71 reduction factor the peak, or does fine-grained local search (0.68–0.73 range) reveal marginal gains? Cycle 0005 showed regression at 0.75, suggesting a narrow optimum.

- **Capacity saturation**: What is the true saturation point for stash capacity relative to workload re-use? The architectural constraint prevents testing 20%, 30%, or higher, leaving the frontier unmapped.

- **Fundamental limits**: After five cycles of incremental refinement within stash capacity and geometric dimensions, gains have become fractional (0.26% in Cycle 0004, null in Cycle 0005). Have we reached the workload-cache pairing's exploitable margin, or do orthogonal levers (workload phase adaptation, alternative collision-reduction strategies, eviction policies beyond FIFO) remain?

## Per-policy notes

- **FIFO insertion (stash)**: Unconditional FIFO insertion into a 25%-capacity stash emerged as the optimal policy. Attempts to layer selectivity on top (filtering by occupancy, collision rate) universally failed, indicating the policy's simplicity is a strength—it imposes minimal overhead while capturing the full re-use window.

- **LRU eviction (stash)**: LRU ordering within the stash showed modest secondary promise relative to FIFO but consistently underperformed orthogonal innovations like geometric rebalancing. The pattern suggests stash-internal ordering is a low-leverage optimization lever once capacity is adequate.
