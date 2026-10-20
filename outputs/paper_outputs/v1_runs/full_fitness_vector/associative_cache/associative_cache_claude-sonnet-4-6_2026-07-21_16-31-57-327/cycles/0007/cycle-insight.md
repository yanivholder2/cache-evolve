# Cycle 0007 Insight

## Cohort outcome

All three variants attempted to decouple the second-chance eviction scan's hit-rate benefit from the 3-swap promotion cost by reducing swaps (to one, then to zero) or by tightening flag placement; none cleared the Winners bar, and v001 was selected as least-bad among a cohort that uniformly regressed.

## Patterns

- **Second-chance scan and 3-swap promotion are mechanically coupled, not independent.** Reducing to one swap or eliminating promotion entirely caused hot items to age into the scan window alongside cold ones, stripping the scan's ability to distinguish them. The eviction quality gain seen in Cycle 0006 is not a property of the flag alone — it requires promotion to maintain hot-item separation from the cold tail.
- **The 3-swap write cost is already below the 100 ns measurement floor.** Eliminating promotion swaps entirely bought no measurable per-op-time improvement, which means the latency regression attributed to "3-swap write pressure" in Cycle 0006's v003 originated elsewhere — the hypothesis that swaps were the bottleneck is now falsified.
- **Flag placement (first-touch vs. position-0 arrival) is neutral on this workload.** Once-accessed mid-range items were not a meaningful drain on eviction capacity under the 8-slot scan window; tightening the threshold produced no measurable hit-rate movement.
- **Compact 32-byte struct loss remains a recurring failure mode.** Variants that overlay only the eviction policy file without explicitly carrying forward the aligned item layout reintroduce per-item footprint bloat and put-path latency — this has now surfaced in multiple cycles and must be treated as a required invariant, not an incidental fix.

## Open questions

- If 3-swap write cost is below measurement resolution, what actually caused the put p99 regression in Cycle 0006's v003 — could it have been an artifact of a different structural change in that variant, such as a scan-window size or a metadata layout detail?
- Is there a write-free way to maintain the hot-item / cold-item separation that the 3-swap loop currently provides, or is positional movement strictly necessary for the second-chance scan to function?

## Continuation

The decoupling hypothesis has been tested exhaustively and rejected — the next cycle should abandon further attempts to separate the second-chance scan from the 3-swap loop and instead treat their combination as a confirmed unit. The productive path is to return to the Cycle 0006 v003 configuration (3-swap + write-once flag + 8-slot second-chance scan + compact 32-byte struct) and diagnose the latency regression source directly, since the swap-elimination experiments have ruled out write pressure as the cause. Compact struct inheritance must be enforced in every variant going forward.
