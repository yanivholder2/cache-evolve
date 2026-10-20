## Recurring wins

- Only one completed cycle so far, so nothing has recurred yet. The one positive signal is that fixed-width bucket repartitioning (v001) was the only variant to improve hit rate, though by a marginal amount.

## Recurring failures

- Hit-rate-only changes have produced little movement. Two of three variants in cycle 0001 left hit rate unchanged, and neither was measured on any other dimension, so their verdicts are incomplete rather than settled.
- Verdicts rest on a single fitness dimension. Latency and metadata size were predicted to matter but were not measured for any variant, so those effects are untested, not refuted.
- Geometry tweaks may be capped by the share of misses they can address. Bucket-conflict misses look like a minor share of total misses at this cache size.

## Open questions

- Does the evaluation path actually select the aging policy, or is its fixed aging period too coarse to change eviction order?
- Does the per-operation cost of scanning wider buckets show up once per-op time is measured?
- Does static dispatch reduce get latency when measured separately from put?
- Which changes can be checked on per-op time and metadata size, so that those dimensions enter the fitness vector?

## Per-policy notes

- Fixed-width bucket repartitioning: the only hit-rate gain seen so far (marginal). Its per-op cost for wider bucket scans is unmeasured.
- Aging LFU: no hit-rate movement observed. Until it is confirmed that the evaluation selects this policy, the result cannot distinguish a weak policy from a path that never exercised it.
- Static-dispatch bucket policy: intended to be neutral on hit rate, so no hit-rate change is expected. Any benefit would be in latency, which has not been measured separately for get and put.
