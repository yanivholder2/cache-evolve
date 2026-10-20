# Cross-Cycle Insights

## Recurring wins

- **Shrinking the small queue to ~10% of capacity** was the single largest gain across all cycles; transferring resident slots from small to main reduced churn and improved hot-item retention without touching admission logic.
- **Ghost queue precision at main_size/2** is a durable independent lever: halving the ghost cap (from the original baseline to main_size/2) improved hit rate on its own, and the gain stacked additively with the enlarged main queue — confirming the two axes are complementary, not redundant.
- **Combining enlarged main + tighter ghost** (cycle 0002 winner) produced the best single configuration jump; each lever amplifies the other because a larger main slows eviction churn, making ghost signal quality matter more.
- **Raising MAX_FREQ from 3 → 4** yielded a real, if small, gain by retaining a genuine tail of hot items that legitimately reach the old ceiling; the direction was confirmed productive before diminishing returns closed it.
- **Move-to-front eviction ordering** is the dominant recency-sorting mechanism in the main queue; its implicit positional churn continuously keeps recently re-accessed items away from the eviction frontier and is load-bearing for the baseline's performance.

## Recurring failures

- **Relaxing the promotion threshold (freq > 0)** was tried twice — once solo (neutral) and once in combination with ghost precision (harmful) — and never improved hit rate; single-access traffic is too sparse and cold on this workload to benefit from wider admission, and the combination dilutes main-queue quality without a compensating gain.
- **Extending the ghost window beyond main_size/2** introduces noise from distant, non-recurring accesses; incidental re-accesses bypass the small-queue filter and degrade main-queue quality.
- **Compressing the ghost window below main_size/2** discards genuine re-use signal; the workload's re-use distances fill the main_size/2 window, making any tighter cap a signal loss.
- **Ghost re-admission credits (seeding re-admitted items at freq=1)** displace established main-queue residents that earned their slots; the ghost signal confirms potential, not enough to justify a positional advantage over incumbents.
- **Positional manipulation within main** (midpoint insertion, saturation-triggered head repositioning) is inert; hot items that reach the frequency ceiling rarely sit close enough to the eviction tail for explicit repositioning to matter.
- **Structural S3-FIFO refinements on workload-a** (eager small-queue promotion, frequency-tournament eviction selection, adaptive threshold control) all failed to improve on the baseline; the mechanisms are sound in principle but find nothing to act on in a Zipfian, phase-stable workload where the existing design already suppresses their target failure modes.
- **Increasing MAX_FREQ beyond 4** yields rapidly diminishing returns; the hot-set tail above the old ceiling is sparse, and each further increment recovers a smaller fraction of eligible items.

## Open questions

- Would adaptive or phase-sensitive mechanisms (e.g., scan-burst detection, dynamic threshold control) improve hit rate on workloads with genuine phase transitions or scan-heavy access patterns, where they would have real signal to act on?
- Is the remaining performance gap between the current best configuration and a theoretical optimum explained by workload-a's inherent access entropy, or is it a structural property of S3-FIFO that an alternative algorithm (TinyLFU, SIEVE, ARC) could exploit?
- Could *conditional* move-to-front — repositioning only above a frequency threshold — extract marginal gains without regressing to the cycle-0005 CLOCK-style failure, given that unconditional repositioning is already the dominant lever?
- Do the null structural results (eager promotion, tournament eviction) generalise to other workloads, or are they specifically a Zipfian property? Testing on a non-Zipfian or scan-heavy trace would determine whether the current design is broadly optimal or narrowly tuned.

## Per-policy notes

- **S3-FIFO — queue sizing:** Small queue at ~10% of capacity is the empirically confirmed near-optimal floor; further reduction beyond 10% shows steeply diminishing returns. Main queue benefits directly from every slot transferred from small. Ghost queue has a hard optimal range centred on main_size/2: both expansion and compression degrade hit rate from opposite causes.
- **S3-FIFO — admission and frequency:** The freq > 1 promotion gate is correct and robust; freq > 0 is safe but inert regardless of what other changes accompany it. MAX_FREQ = 4 is the practical ceiling for this workload; the ceiling direction is closed. Second-chance eviction cannot compensate for upstream admission noise — it depends on the frequency and ghost guards being correct first.
- **S3-FIFO — eviction ordering:** Move-to-front is not a cosmetic detail; it is the primary recency-sorting mechanism and its removal causes large regressions. Fine-grained positional refinements on top of it are uniformly inert. The policy's performance on workload-a appears to have reached a local optimum: all plausible levers across seven cycles have been mapped to diminishing returns or confirmed failures.
