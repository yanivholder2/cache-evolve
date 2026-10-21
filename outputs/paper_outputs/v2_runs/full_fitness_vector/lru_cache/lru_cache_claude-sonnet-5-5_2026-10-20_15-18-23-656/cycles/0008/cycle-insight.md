# Cycle 0008 Insight

## Cohort outcome

Three low-risk refinements of the cycle-0007 ghost-gated baseline were tried: a halved ghost table (v001), a larger protected share of the segmented cache (v002), and fewer redundant hashes in `put` (v003, ghost lookup gated on a full cache and the victim hashed once). All three were Winners. The gains were small. The ghost-cost work reduced put tail latency, and the constant tweak gave a tiny hit-rate gain.

## Patterns

- Ghost overhead can be trimmed without losing the scan-filter benefit. A smaller table (v001) and less hashing (v003) both cut put tail latency while hit-rate held. The cycle-0007 ghost design was oversized and redundant.
- Both ghost-cost wins reach only the put tail. The median put is dominated by hash plus map insert, and `get` is untouched. Further gains need structural changes, such as fewer map operations per put, not fewer hashes in a fixed sequence.
- The protected-share trend still gives hit-rate gains, now on top of ghost-gated insertion, but they are tiny. It remains a weak, low-risk knob with diminishing returns.
- Per-op-time deltas again appeared on paths the change cannot touch (get after a ghost-only edit, get p50 after a constant-only edit). This is noise, so small timing deltas stay unattributable.
- The metadata-size metric is blind to heap-allocated ghost memory, so the claimed memory savings are unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- How far can the ghost table shrink, or become sampled, before hit-rate degrades? The current evidence only shows that halving is tolerable.
- Can the number of map operations per put be cut, to move the median put and not just the tail?
- Does the ghost-gated design, with its larger protected share, hold up on non-scan workloads? This check has been pending for many cycles.
- What is the per-op-time noise floor? Repeat-run measurement is still missing.

## Continuation

Treat the v001 and v003 combination (smaller ghost table plus single-hash gated eviction path) as the baseline. Pursue structural put-path reductions, such as fewer map operations, and add a heap-inclusive memory measure so ghost cost is visible. Drop further protected-share constant steps and hash or clone micro-optimizations, since they are saturated or within noise. Evaluate on non-scan workloads and measure a noise floor before crediting small speed claims.
