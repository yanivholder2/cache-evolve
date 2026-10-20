# Variant v002 Insight

## Helped

- per-op-time: Gating `evict_small()` behind the admission sketch for first-seen cold keys when `small` is full cut put p50 in half and reduced put p99, confirming that skipping the eviction call for one-hit-wonder keys is the primary lever on put latency when the small queue is tiny.

## Hurt

- hit-rate: Rejecting first-seen cold keys under pressure filtered out a fraction of keys that are eventually re-accessed on the workload-a trace, producing a small but measurable hit-rate regression — meaning the one-hit-wonder assumption does not fully hold for this Zipfian workload.
- metadata-size: The three new fields (`admit_sketch`, `evict_count`, `sketch_seed`) grew the `S3Fifo` struct from 256 to 280 bytes, adding constant per-cache overhead regardless of capacity.

## Takeaway

The sketch successfully reduces `evict_small()` invocations and delivers measurably lower put latency, but the hit-rate cost reveals that workload-a contains enough "warm cold" traffic — keys that are cold at first admission yet re-accessed soon after — that a strict first-seen rejection policy is not hit-rate neutral. A two-visit threshold (require count ≥ 2 before admission) or a shorter sketch reset interval could reduce this false-rejection rate without giving up the latency gain.
