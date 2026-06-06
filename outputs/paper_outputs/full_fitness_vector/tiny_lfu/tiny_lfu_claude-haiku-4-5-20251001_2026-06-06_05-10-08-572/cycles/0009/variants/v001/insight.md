# Variant v001 Insight

## Helped

- hit-rate: Expanding victim selection from 1 candidate to 3 (LRU head + 2-iteration scan) enables the algorithm to identify colder items deeper in the LRU queue, avoiding eviction of hotter items and improving cache discrimination (0.7243 → 0.727).
- get p99: Improved get tail latency is a downstream benefit of better hit-rate behavior and cache stability from more precise eviction decisions.

## Hurt

- put p50: Evaluating 2 additional candidates in the victim selection loop adds ~10ns to median put latency as the algorithm traverses more LRU nodes and invokes `estimate()` more frequently during cache-full scenarios.
- put p99: Worst-case put latency increases by ~119ns (271 → 390) because the extended candidate scan runs its full 2-iteration loop under eviction pressure, incurring multiple sketch lookups and frequency comparisons.

## Takeaway

Expanding the frequency-aware victim pool from N=2 to N=3 yields a modest hit-rate gain (0.37%) at a measurable but tolerable latency cost. The 43% p99 increase on put is significant but remains an order of magnitude below the catastrophic spikes seen at N=5, confirming that this middle ground is viable. The change proves that the eviction pool parameter is exploitable: moving beyond pure LRU + single candidate still captures diversity gains without the latency pathology of deeper scans, validating the hypothesis that N=3 occupies a sweet spot between accuracy and performance.
