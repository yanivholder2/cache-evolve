# Variant v003 Insight

## Helped

- hit-rate: Replacing recency-ordered victim choice with a 2-bit RRPV per slot, inserted at 2 and cleared to 0 on hit, lets re-referenced items outlast one-shot inserts that age out first, which is the intended source of the workload-a gain.

## Hurt

- per-op-time: The RRPV write on every get hit adds a small fixed cost to the get path, and the bounded aging scan on a full-bucket put adds tail latency to put, so the put p99 tail is the largest regression.

## Takeaway

Giving each slot a multi-level re-reference value rather than a single recency or second-chance bit moved hit-rate in the direction the hypothesis predicted, but the gain comes from one trace at one cache size. The cost sits in the put tail, because a full-bucket victim search can run up to three aging rounds, so any follow-up should measure put p99 before stacking more retention logic on this policy.
