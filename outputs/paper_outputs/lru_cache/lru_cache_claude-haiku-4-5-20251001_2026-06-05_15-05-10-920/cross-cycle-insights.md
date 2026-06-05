# Cross-Cycle Insights

## Recurring wins

- **Frequency-based eviction protecting hot items**: Consistent across cycles 0001–0010. Eviction policies that rank items by access frequency and protect frequently-accessed items from eviction outperform pure recency (LRU) and zone-partitioning approaches. The workload exhibits access-skewed patterns where protecting hot items is more important than uniform temporal ordering.

- **Periodic decay at fixed interval (capacity × 16)**: Introduced in Cycle 0002, locked in through Cycle 0009, and confirmed robust in Cycle 0010. All attempts to accelerate (capacity × 8), decelerate (capacity × 24), make adaptive (Cycle 0009 v001/v003), or change curve shape (Cycle 0005 v003) have regressed or stagnated. This interval balances frequency accumulation against phase adaptation; deviations in either direction break the calibration.

- **Simplicity over complexity**: Multiplicative decay (0.75× per cycle) outperforms exponential approximations (Cycle 0005 v003). Uniform decay beats selective exemptions (Cycle 0007 v002). Fixed-interval decay beats activity-responsive decay (Cycle 0009 v001). Uniform mechanisms avoid the per-operation overhead and frequency-signal corruption that adaptive approaches introduce.

- **Two-pass LRU victim selection**: Introduced in Cycle 0006, confirmed in Cycles 0007, refined in Cycle 0008. Selecting the head-most (oldest) item among those with minimum frequency captures temporal locality within frequency tiers more accurately than single-pass or recency-mixed approaches.

- **Full-precision frequency tracking**: Cycle 0010 v002 reveals that 8-bit frequency counters outperform 3-bit bucketed quantization, reversing Cycle 0009's "neutral" assessment of bucketing. Precision is not a negligible micro-tuning lever; losing granularity to compression genuinely degrades victim selection.

## Recurring failures

- **Zone-based cache partitioning**: Attempted in Cycles 0001–0003 (separate cold/hot zones, independent decay rates, resized partitions). All regressed or underperformed unified frequency-based eviction. Partitioning introduces promotion barriers and masks frequency signals across zone boundaries; items with moderate frequency get evicted from cold zones before earning hot-zone protection. Unified competition on frequency alone is superior.

- **Adaptive or dynamic mechanisms**: Cycle 0003 v002 (distribution-aware thresholds), Cycle 0005 v003 (exponential curves), Cycle 0007 v002 (selective decay), Cycle 0009 v001 (activity-responsive decay), and Cycle 0009 v003 (hit-rate monitoring) all regressed. Adaptive tracking adds per-operation overhead without corresponding gains, suggesting either the workload lacks sharp phase transitions or the cost of detecting them outweighs the benefit of earlier triggering.

- **Hybrid frequency-recency scoring**: Cycle 0007 v003 showed promise when tested in isolation, but Cycle 0008 tested it in combination with the decay mechanism: all three variants (100× coefficient with two/one-pass selection, 25× coefficient) regressed uniformly. Blending recency into frequency-based eviction scores introduces noise that overrides frequency discrimination. The decay mechanism has already calibrated frequency values to optimal ranges; adding positional bias inverts the eviction ordering.

- **Selective decay exemptions for weak frequencies**: Cycles 0003 v002, 0006 v003, 0007 v002 all attempted to shield low-frequency items from decay or from eviction. All regressed or failed. Shielding weak signals allows them to retain artificial inflation and degrade the cache's ability to discriminate between ephemeral and persistent access patterns. Either commit fully to threshold-based signal rejection (Cycle 0010 v003 amnesia succeeds) or apply uniform decay to all.

- **Accelerating the decay interval**: Cycle 0004 v002 (capacity × 8) and Cycle 0006 v001 (capacity × 24) both regressed. The capacity × 16 interval is not one point in a tunable continuum; it is robustly optimal for this workload's phase-shift rate.

## Open questions

- **Can Cycle 0010's two improvements combine?** v002 (8-bit precision) and v003 (threshold amnesia for freq < 2) both improved hit-rate independently. Testing their combination would reveal whether they address orthogonal inefficiencies or compete for the same optimization budget. If additive, this becomes the new baseline; if competitive, confirm which is dominant.

- **What is the optimal threshold for frequency amnesia?** Cycle 0010 v003 used freq ≥ 2 as the cutoff. Is this boundary tuned to this workload, or can it be optimized further? Threshold sensitivity analysis (freq ≥ 1, freq ≥ 3, etc.) could clarify whether 2 is a local optimum or whether the marginal gain is robust to threshold position.

- **Why does threshold amnesia succeed where selective decay failed?** Cycles 0003, 0006, 0007 attempted to exempt weak frequencies from decay or eviction; all regressed. Cycle 0010 v003 applies sharp amnesia (complete discard for freq < 2) and improves. The structural difference—complete rejection vs. partial exemption—deserves investigation. Does amnesia work because it avoids the hybrid-scoring corruption, or because it removes low-confidence signals entirely?

- **Has the optimization window closed, or is there room beyond precision and thresholding?** After 5 cycles of decay tuning, 2 cycles of victim-selection refinement, and 1 cycle of revisiting precision, the cache may be approaching saturation. Cycle 0010's recovery suggests the plateau was misdiagnosed, but 2–3 more incremental wins may exhaust the margin. Does the workload contain further exploitable structure (e.g., multi-tier thresholds, frequency quantization refinement), or have the major gains been captured?

## Per-policy notes

- **Frequency-based eviction**: Dominant policy across all cycles. Outperforms pure LRU (Cycle 0010 v001 decisively hurt), zone-based partitioning (Cycles 0001–0003), and ghost-list mechanisms (Cycle 0001). The workload's access patterns are sufficiently skewed that protecting hot items is load-bearing. Pairs cleanly with multiplicative decay and LRU tie-breaking but gets corrupted by hybrid recency scoring.

- **Multiplicative decay (0.75× at capacity × 16 intervals)**: Optimal decay mechanism, locked in from Cycle 0004 through Cycle 0009 with high confidence. Coefficient adjustments (0.7×, 0.8×, exponential approximations) regress; interval adjustments (capacity × 8 or 24) regress. The mechanism is not locally optimal but globally robust for this workload. Interacts cleanly with full-precision frequency tracking but conflicts with hybrid recency-weighted scoring, suggesting the decay mechanism has already fully calibrated frequency values.

- **Victim selection via two-pass LRU**: Superior to one-pass selection (Cycle 0008 v002 regressed vs. v001) and pure temporal ordering. Selecting the oldest item among minimum-frequency candidates captures real temporal locality within frequency tiers. Tie-breaking precision is no longer a strong lever once decay and frequency tracking are optimal; this mechanism appears near saturation.

- **Zone-based and partitioned policies**: Consistently underperform across Cycles 0001–0003. Introducing zone boundaries creates promotion barriers and masks frequency signals; unified competition on frequency alone is superior. Do not revisit unless entirely different workload characteristics emerge.

- **Precision and quantization**: Full 8-bit frequency tracking superior to 3-bit bucketing (Cycle 0010 v002). Counter saturation not a constraint (Cycle 0004 v001 neutral). Precision loss is subtle but real; bucketing compressed frequency range without benefit.

- **Threshold mechanisms**: Early selective-exemption approaches fail (Cycles 0003, 0006, 0007). Sharp frequency amnesia succeeds (Cycle 0010 v003). The distinction is critical: exempting weak signals from decay allows noise to accumulate; completely discarding them avoids noise while reducing false positives. Thresholds only work if combined with definitive signal rejection, not partial protection.
