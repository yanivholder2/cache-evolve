## Recurring wins

- Admission-side sizing is the only lever with consistent signal: a small probation queue (a few percent of capacity, well below the baseline) paired with a shortened ghost beats baseline across several cycles (0002, 0003, 0006, 0007, 0009).
- Sizing alone explains the gains. Later bundles (main-evictee ghosting, hit-path promotion) added nothing once sizing was held fixed.
- Ghost length acts as an admission filter: shorter (stricter) helps, longer hurts.
- Simplest configuration wins ties: the sizing-only config matches the more complex variants.

## Recurring failures

- Ageing/decay changes to main (faster freq decay, lower freq cap, in-place ageing) lose every time. This workload rewards keeping reuse history. A higher cap gave at most noise-level gain.
- Rewrites of main-eviction mechanics (lookahead, reinsertion caps) lose. The unbounded second-chance scan does protective work.
- Loosening admission (looser promotion gate, pre-protected or boosted ghost re-admits, grace passes) loses by polluting main.
- Filtering ghost hits or ghost content (freq>0 fast path, reuse-only ghosting) loses. Ghost hits are strong reuse evidence.
- Promotion strictness/timing (eviction-time gate, hit-path promotion, thresholds, freq reset) ties parent repeatedly. These are either non-binding or no-op diffs.
- Adaptive/feedback sizing lost to the fixed ratio.
- Per-key ghost metadata and main insertion-position tweaks gave no signal.

## Open questions

- Where is the small-queue optimum? 3% vs 5% vs larger were never compared head to head on a shared parent.
- Are exact ties real or no-op diffs? Variant authors cannot read overlay source, so behaviour change is unverified.
- Do the sizing gains hold on a second trace and cache size, including scan-heavy workloads? All evidence is one trace at one size, and gains are small and may be noise.
- Does ghosting main evictees add anything on top of sizing?
- Per-op-time and metadata-size have never been measured, so cost is unknown, especially for hit-path work and extra ghost inserts.

## Per-policy notes

- S3-FIFO (the only policy so far): baseline structure looks near a local optimum on this trace. Remaining untested ideas: a sweep of the small ratio, validation on other traces, and structurally new levers that are verified to change behaviour.
- Drop promotion thresholds, ageing/decay, adaptive sizing, ghost-content filters, and main-eviction rewrites from future hypotheses.
