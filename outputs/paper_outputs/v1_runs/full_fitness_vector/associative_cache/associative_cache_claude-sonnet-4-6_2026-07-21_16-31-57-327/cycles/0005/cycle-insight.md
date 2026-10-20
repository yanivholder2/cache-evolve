# Cycle 0005 Insight

## Cohort outcome

All three variants probed promotion strategy and frequency-signal placement: one extended the unconditional swap budget, one added a frequency overlay at the eviction boundary, and one gated promotion distance on an access counter. Only the purely positional, write-free approach (v001) landed in Winners.

## Patterns

- **Unconditional multi-swap promotion wins cleanly at no latency cost.** The 3-swap loop's early-break keeps the hot-path at one iteration for already-head items, so Zipfian workloads see the full convergence benefit with no measurable overhead. Promotion budget expansion from one to three swaps was the right lever; the convergence lag at single-step is real and now resolved.
- **Frequency signals on the get path carry consistent tail-latency risk.** Both variants that wrote `lfu_counter` per access surfaced p99 pressure — v002 from counter writes hurting eviction ordering, v003 from the combined weight of counter writes plus up to three additional swaps. The write itself, not the conditional logic, appears to be the cache-line liability on the get path.
- **Frequency overlays at the eviction boundary degrade hit rate on this workload.** Two-tail frequency comparison (v002) overrode clean positional recency, ejecting recently-promoted items before genuine LRU victims. Strict tail eviction remains the correct policy; no frequency signal applied at eviction time has improved upon it across two cycles of testing.

## Open questions

- Does promotion budget beyond three swaps (e.g., four or five) continue to gain hit rate, or have diminishing returns already set in at three given the Zipfian concentration of hot keys near position 0?
- v003's p99 doubling is the first latency signal to clearly clear the measurement floor — does it indicate a general threshold on per-get write volume, and can that threshold be quantified to bound future frequency-integration experiments?

## Continuation

The next cycle should build on v001's 3-swap unconditional loop as a confirmed base and investigate whether frequency information can be incorporated without any write on the hot get path — for example, sampled or epoch-based counters that update only on eviction or put, not on every get. Two-tail eviction and any per-get counter write should be treated as closed unless a fundamentally different encoding is proposed. If hit rate appears to have plateaued after further promotion-budget trials, the focus should shift to understanding the measurement floor more precisely, as v003's p99 signal suggests latency differentiation is now observable under write-heavy get paths.
