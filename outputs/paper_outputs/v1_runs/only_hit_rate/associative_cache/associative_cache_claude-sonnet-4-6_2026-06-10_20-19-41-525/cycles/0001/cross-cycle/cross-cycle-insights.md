## Recurring wins

- **Wider bucket geometry improves hit-rate.** Consolidating the same total capacity into fewer, larger buckets gives each per-set LRU a bigger eviction pool and reduces premature expulsion of live items. Confirmed once on the LIRS workload-a trace; not yet tested on other workloads.

## Recurring failures

- **Naive O(1) LRU promotion (swap-to-back) degrades hit-rate.** Preserving only the head/tail boundary indices while corrupting the intermediate recency gradient produces a measurable hit-rate regression. The O(n) cost of strict ordering is worth paying for ClassicLRU.
- **Hypotheses that lack matching evaluators go unresolved.** The metadata-compaction variant could not be confirmed or denied because the metadata-size and per-op-latency evaluators did not run — wasted variation budget.

## Open questions

- Does the bucket-widening gain generalise beyond the LIRS workload-a locality pattern, or is it trace-specific?
- Is there a recency-preserving O(1) promotion strategy (e.g., clock approximation, segmented LRU) that avoids the gradient corruption seen with plain swap-to-back?
- Can metadata-size and per-op-time evaluators be activated so structural compaction changes get a real verdict?
- Is there a saturation point for bucket widening, and if so, does adaptive bucket sizing outperform any fixed geometry?

## Per-policy notes

- **ClassicLRU:** Strict recency ordering is critical — any shortcut that collapses the gradient hurts. Bucket geometry is a stronger lever than per-op micro-optimisations on this workload. Timestamp fields are unused by the eviction path, making metadata compaction plausible but currently unmeasured.
