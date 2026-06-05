# Cross-Cycle Insights

## Recurring wins

- **Two-segment SLRU as a foundation**: Inserting new entries into a probationary segment before they can displace proven hot entries has delivered durable gains every cycle it appeared. Acting at insertion time (rather than eviction time) is the root of this advantage.
- **Scan-fraction sizing over fixed counts**: Mechanisms that scale their working-set scan to a proportional share of the segment (e.g., one quarter of protected) consistently outperform those using a small fixed count, because the signal stays meaningful as capacity grows.
- **Counter halving over counter reset on demotion**: Preserving a decayed form of accumulated frequency history across segment transitions allows promoted entries to remain distinguishable from newcomers — resetting to zero throws away a durable and informative signal.
- **Position-update discipline on protected hits**: Every mechanism that maintained continuous move-to-tail on access in the protected segment produced better ordering; suppressing position updates for any reason has consistently degraded hit rates.

## Recurring failures

- **Eviction-time-only filters**: Both the CLOCK bit-scan (cycle 1) and the fixed-window frequency scan (cycle 1) acted only when eviction was forced, missing entries that should have been deprioritized much earlier. Eviction-time intervention is weaker than insertion-time or promotion-time routing.
- **Small fixed frequency windows**: A constant small look-ahead (e.g., 8 entries) leaves a structural ceiling on improvement regardless of how accurate the frequency signal is within that window.
- **Ghost admission on this workload**: Tested in isolation in cycle 3 after cycle 2's ambiguity, the ghost set produced negligible lift. The probationary-capacity ghost window is too narrow to intercept reuse gaps in the current trace; the improvement originally attributed to it was SLRU segmentation.
- **Protected-segment widening past 80/20**: Shrinking the probationary share to 30% triggered more demotion churn than the extra protected headroom could absorb; the hot working set is large enough that capacity allocation shifts hurt more than they help.
- **Second-chance reprieve with position freeze**: Granting a one-time demotion reprieve while freezing node position severed the recency ordering that makes LRU useful. Hot entries stalled near the demotion head regardless of their reprieve status.

## Open questions

- Does frequency-aware demotion interact cleanly with ghost admission, or does the ghost set introduce noise now that both operate simultaneously in the same pipeline?
- Is the LRU-quarter scan fraction robust across different workload shapes, or does the right fraction shift with reuse-distance distributions?
- Could a significantly larger ghost window (e.g., equal to total capacity rather than probationary capacity) make cross-residency warmth visible enough to matter on this trace, or is the ghost concept fundamentally mismatched to the workload?
- Could a second-chance approach be rehabilitated by setting the referenced bit while still performing move-to-tail, so recency ordering is preserved alongside the reprieve?
- Does CLOCK's linked-list-free structure yield measurable throughput gains under high concurrency or at very large capacities, separate from its hit-rate characteristics?

## Per-policy notes

- **Plain LRU (baseline)**: Scan-pollution vulnerability is confirmed real and addressable by multiple independent mechanisms. All variants in cycle 1 improved on it.
- **SLRU (v001, cycle 1 winner)**: Probationary-to-protected promotion is the load-bearing mechanism. The 80/20 protected/probationary split performed best; both narrower and wider protected segments degraded results. This structure has served as the base for all subsequent winners.
- **CLOCK (cycle 1)**: Eliminates pointer maintenance overhead; hit-rate gains were real but smaller than SLRU because eviction-time intervention is weaker. Throughput advantage under concurrency remains untested.
- **Frequency-window eviction (cycle 1 v003, cycle 2 v002 retry)**: Cycle 1's fixed-8-slot version had a structural ceiling. Cycle 3's redesign — proportional scan window plus counter halving — became the winner, confirming the idea is sound once signal-design flaws are corrected.
- **Ghost admission (cycle 2 v003, cycle 3 v001 isolation)**: Appeared effective in cycle 2 but was confounded with SLRU. When isolated in cycle 3, contribution was negligible on this workload. Not worth pursuing unless workload shifts toward a trace with frequent cross-residency reuse.
- **Frequency-aware demotion with counter halving (cycle 3 winner)**: Now the strongest single add-on to SLRU. Scanning the LRU quarter of protected and halving (not resetting) counters on demotion are both required for the mechanism to work; either alone was insufficient in earlier designs.
