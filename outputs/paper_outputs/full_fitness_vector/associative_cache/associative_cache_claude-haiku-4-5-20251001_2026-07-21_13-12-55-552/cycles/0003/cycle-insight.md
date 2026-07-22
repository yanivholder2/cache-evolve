# Cycle 0003 Insight

## Cohort outcome

This cohort tested the bounds of cache geometry: narrower buckets to reduce scan depth, a policy swap (FIFO) to avoid per-hit bookkeeping overhead at wider bucket sizes, and even wider buckets to push the hit-rate improvement trajectory. Narrower geometry (256×8) regressed both hit-rate and latency. The FIFO policy variant failed to execute its intended change, leaving the latency-improvement hypothesis untested. Wider geometry (32×64) extended hit-rate marginally but caused tail latencies to spike significantly, signaling an inflection point in the geometry trade-off space.

## Patterns

- **Geometry trade-off is saturating**: Cycle 0002's wider buckets (64×32) yielded hit-rate gain (+0.24%) at latency cost (-45%). Cycle 0003's even wider buckets (32×64) produced minimal hit-rate gain (+0.0005%) while tail latencies degraded sharply (p99 doubling), indicating diminishing returns and accelerating latency penalty as bucket width increases.
- **ClassicLRU's O(n) scan overhead is the bottleneck at scale**: Both v002 and v003 demonstrate that intra-bucket linear search becomes increasingly expensive as bucket size grows. The sharp tail-latency spike in v003 suggests that outlier scans on full buckets are dominating percentile performance, a micro-architectural constraint that narrower buckets avoid but at other costs.
- **Narrow buckets backfire despite theory**: The prediction that reducing scan depth (16→8 items) would improve latency was contradicted by v001's 45% regression. Hash-function distribution overhead and memory-layout effects across 256 buckets outweigh any benefit from shallower scans, repeating the pattern from prior cycles where theoretical asymptotic gains fail in practice.
- **Policy pivot was blocked**: H02's hypothesis—that FIFO's zero per-hit bookkeeping would decouple latency from bucket width—was sound but the variant either failed to compile the policy change or it was overridden. This leaves the testable claim unresolved and suggests a build/integration issue.

## Open questions

- Why did v002 (H02) not execute the FIFO policy change despite the hypothesis specifying it? Was this a build failure, override, or integration issue that could be fixed in the next cycle?
- If FIFO were correctly implemented on 64×32 geometry, would it flatten the latency cost while retaining the hit-rate gain, or does the policy change require additional structural changes?
- Is the tail-latency spike in v003 an inherent worst-case of linear bucket search, or an implementation artifact (e.g., cache misses, branch mispredictions) that could be mitigated?
- Has the cache geometry reached a local optimum, or does the fitness landscape favor wider buckets at different workload distributions or sizes?

## Continuation

The cohort has identified a clear frontier: wider buckets improve hit-rate but at an accelerating latency cost under ClassicLRU's per-hit movement operations. The next cycle should: (1) resolve the H02/FIFO integration issue and re-run v002 with the intended policy change to test whether decoupling latency from bucket width is feasible, (2) if FIFO succeeds, probe the local optimum around 64×32 FIFO to confirm it is a fitness win, or (3) if FIFO fails or shows no improvement, accept that ClassicLRU + 64×32 is the current Pareto frontier and shift focus to orthogonal avenues: robustness across different workload distributions, multi-core contention, or policies designed for large-bucket efficiency (e.g., segment-based or batched eviction).
