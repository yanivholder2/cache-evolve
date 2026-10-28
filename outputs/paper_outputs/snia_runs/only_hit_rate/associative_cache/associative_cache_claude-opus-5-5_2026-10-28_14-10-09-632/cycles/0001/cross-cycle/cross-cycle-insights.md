## Recurring wins

- (Only 1 cycle so far, so nothing is recurring yet. These are early signals.)
- The winning lever was where new items get inserted. Cold, LIP-style insertion below reused items raised hit-rate on every MSR trace×size cell. What decides hit-rate is how new items rank against re-referenced ones.
- Coarse recency (CLOCK second-chance) kept hit-rate level, slightly better at small size. Exact LRU order does not seem to drive hit-rate.

## Recurring failures

- Weakening hit promotion while new items still enter at MRU lowers hit-rate. New items end up above reused ones. hm_0 at the larger size took the biggest hit.
- A probation window set as a fixed fraction of the bucket is too short in small (128-slot) buckets and cuts off real reuse there. The gain is concentrated in larger buckets.
- Cost dimensions (per-op-time, metadata-size) were not measured. Latency claims are still untested.

## Open questions

- Does CLOCK actually lower get/put latency, given that it costs no hit-rate?
- Would a deeper insertion point, or a probation depth that scales with bucket size, recover hits at the smaller cache size without losing the gain at the larger one?
- Does CLOCK combined with cold insertion keep both benefits?
- Does graded or bounded promotion help once inserts already land low?

## Per-policy notes

- ClassicLRU + LIP-style insertion (len/4): current best. Strongest with large buckets, slightly too aggressive with small ones. The insertion fraction is the tuning knob.
- ClassicLRU + bounded promotion: hurts on its own. Revisit only together with cold insertion.
- CLOCK second-chance: neutral to slightly positive on hit-rate. Latency benefit still unmeasured. A candidate base for combining with cold insertion.
