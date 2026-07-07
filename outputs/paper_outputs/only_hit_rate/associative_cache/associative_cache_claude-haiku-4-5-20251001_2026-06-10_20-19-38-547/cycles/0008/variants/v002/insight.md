# Variant v002 Insight

## Helped

(None measured in this evaluation)

## Hurt

- **hit-rate**: Inserting a mandatory warm-tier intermediate stage between cold and hot stash forces items to survive two re-accesses before reaching premium hot-tier space, losing single-re-access items that were previously promoted immediately when buckets were full.

## Takeaway

The three-tier staged-promotion model (cold → warm → hot) underperforms the two-tier immediate-promotion approach on this workload. By requiring frequency proof (two re-accesses) before committing to the hot tier, the design filtered out transient items as intended, but at the cost of delaying promotion of genuinely useful single-re-access items. The workload's access pattern appears to reward rapid escalation over observation; items that demonstrate one re-access are better served by immediate hot-tier placement than by staged gatekeeping.
