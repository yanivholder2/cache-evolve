# Cycle 0003 Insight

## Cohort outcome

All three variants executed exactly the directions suggested by the previous cycle's continuation — composing 2Q with the MRU-tail guard, tuning the protected-queue quota, and eliminating key-clone cost on eviction — yet none produced a clean Winner; every variant either regressed latency or improved hit-rate at the cost of higher per-op time.

## Patterns

- **The MRU-tail guard does not generalize across workloads.** Cycle 0002's guard delivered a win on its own workload, but when composed with the 2Q layout on workload-a, the guard fires too rarely: the branch and extra pointer read add cost on every protected hit while the bypass only triggers for the rare exact-tail case. Guard-style shortcuts only pay off when their fire rate is high enough to amortize the check.
- **Protected-quota tuning is a genuine but costly lever.** Expanding the protected budget from 75% to 87.5% of capacity measurably reduced demotion pressure and improved hit-rate on a Zipfian trace, confirming the hypothesis was directionally correct. The price is proportionally more expensive unlink/push_back_prot operations, meaning quality and throughput move in opposite directions as the quota shrinks.
- **Clone-elision must be validated against the actual key type.** The key-clone optimization had no effect because the benchmark uses a `Copy` key where `.clone()` is a register copy. The idea remains sound for heap-owning key types but cannot be confirmed or denied by the current trace setup.
- **No composition of the cycle-0002 winners produced a net win.** The two prior winners targeted orthogonal bottlenecks, but layering them introduced cumulative branch and path-length overhead that outweighed the individual gains on this workload.

## Open questions

- Is there a workload or access pattern where the protected-quota expansion's hit-rate gain dominates its latency cost, making v002 a net winner by a different metric weighting?
- Can the guard fire rate for the MRU-tail shortcut be increased by extending it to the probationary tail as well, or is workload-a simply too diffuse for any single-entry bypass to matter?
- What is the correct benchmark fixture to evaluate key-clone elision — a `String` or `Arc<str>` key trace — and does the eviction-path refactor actually reduce allocations there?

## Continuation

The next cycle should step back from composition and micro-tuning and instead probe structural changes that are independent of the current 2Q + guard stack. Priority directions: (1) evaluate whether adaptive quota sizing (dynamically shifting the protected/probationary boundary based on observed hit rates) can capture the hit-quality benefit of v002 without permanently enlarging the more-expensive protected path; (2) benchmark the key-clone elision against a non-`Copy` key type before deciding whether to carry it forward or discard it. Do not revisit the MRU-tail guard in isolation — its fire rate is workload-dependent and the current trace does not justify the check cost.
