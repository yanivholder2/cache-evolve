# Cross-Cycle Insights

## Recurring wins

- **Frequency-based protection**: Every cycle builds on protecting frequently-accessed items via frequency tracking as the core eviction signal. This insight from Cycle 0001 has remained the structural foundation through all six cycles; no cycle abandoned it, and every winner variant preserves frequency-driven selection.

- **Fixed, simple thresholds over adaptive complexity**: Cycles 0002–0006 consistently show that `capacity × 16` reset interval, fixed 0.75× multiplicative decay coefficient, and deterministic tie-breaking rules outperform attempts at dynamic tuning. Adaptive mechanisms (Cycle 0003 v002, Cycle 0005 v003) add computational overhead without proportional gains.

- **Decay curve shape dominates decay interval**: Cycle 0004's win with multiplicative 0.75× over right-shift division, confirmed by Cycle 0005's coefficient plateau, reveals that *how steeply* the cache forgets matters more than *how often* it resets. Gentler, multiplicative forgetting preserves frequency discrimination across phase shifts better than aggressive right-shift.

- **Orthogonal precision yields gains when mechanics plateau**: Cycle 0006's two-pass LRU tie-breaking win after five cycles of exhausting decay-mechanism tuning shows that once a signal is tuned, the bottleneck shifts to eviction *precision*. Victim-selection refinement is the natural frontier once frequency decay is locked in.

## Recurring failures

- **Zone-based partitioning and cache splitting**: Cycles 0001–0003 show repeated attempts to split the cache by access pattern (ghost lists, cold/hot zones, hybrid partitioning) consistently underperform or at best match unified frequency-based competition. Artificial zone boundaries mask frequency signals and create unfair promotion barriers; unified frequency competition is fundamentally stronger for this workload.

- **Tie-breaking direction reversal**: Cycle 0002 v001's attempt to reverse LRU scan direction (preferring newer low-frequency items over older ones) degraded performance, confirming that head-to-tail LRU ordering correctly captures temporal locality within equal-frequency groups.

- **Coefficient and interval perturbations without structural change**: Cycles 0004–0005 establish that the current decay tuning (0.75× at capacity × 16) sits at a local optimum; any deviation—faster decay, slower decay, gentler decay, more aggressive decay, longer intervals, shorter intervals—regresses or stagnates. This plateau is fundamental, not a missed optimization.

- **Complexity that adds measurement overhead**: Cycle 0003 v002's adaptive frequency-distribution-aware decay and Cycle 0005's exponential approximation both introduced tracking or computation overhead that outweighed benefits. Simpler signal (static interval, fixed coefficient) is superior.

## Open questions

- **Tie-breaking refinement scope**: Cycle 0006 v002 gained from two-pass LRU selection, but is there room for further orthogonal tie-breaking? Could frequency-recency hybrids, decay-adjusted scoring within frequency tiers, or other heuristics push the boundary further?

- **Threshold-dependent decay**: Cycle 0006 v003 (selective decay for frequencies ≥ 4) failed due to backend error but remains unexplored. Could applying different decay rates to hot vs. cold items, or skipping decay for low-frequency entries, compound with the two-pass approach?

- **Frequency quantization and discrimination**: Does rounding or bucketing frequency values sharpen or blur discrimination among hot items? Could coarser quantization reduce counter overhead while preserving the frequency ordering the cache relies on?

- **Workload phase-shift properties**: What makes this workload sensitive to decay curve shape? Is the frequency distribution multimodal, creating sensitivity to forgetting rates, or do gradual phase shifts benefit from preserving temporal patterns longer?

## Per-policy notes

- **Frequency-based eviction (Cycles 0001–0006)**: Consistently strongest. Optimal configuration: 0.75× multiplicative decay per `capacity × 16` reset interval, with two-pass LRU victim selection (find minimum frequency, then evict head-most item with that frequency). This configuration locks in after Cycle 0006 as no further decay-mechanism gains are available.

- **Zone-based and hybrid approaches (Cycles 0001–0003)**: Repeatedly regressed or failed to match unified frequency approach. Cold/hot zone splitting, ghost-list re-insertion, and zone-aware decay all introduce artificial barriers that undermine the core advantage of unified frequency competition. Should not be revisited.

- **Decay mechanics exploration (Cycles 0002–0005)**: Fully exhausted. Both interval (capacity × 8 through × 24) and coefficient (0.7× through 0.8×) sensitivity testing confirms plateau at current tuning. Exponential and other non-linear decay patterns offer no advantage. This mechanism is stable and locked; future gains must come from orthogonal precision levers.
