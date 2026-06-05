## Recurring wins

- **Hot-tier topology expansion** drove the largest hit-rate gains in cycles 1–2; redistributing capacity toward the normal (hot) tier consistently outperformed any policy change made at the same time. The gain curve peaked at 24 normal / 8 stash and has since been confirmed exhausted.
- **MRU insertion positioning** is universally winning on workload-a; placing every item — whether newly inserted, promoted, or re-admitted — at the MRU end consistently preserves hits across all cycles tested. No non-MRU positioning variant has improved on this.
- **Stash traversal as an admission filter** is load-bearing; every approach that weakened or bypassed it (direct insertion, ghost-bypass on cold items, LFU-gated promotion) regressed. Preserving full stash traversal is now a confirmed constraint, not a candidate for optimisation.
- **Compounding additive axes** (topology + MRU re-entry together) reliably outperforms either axis alone, even when individual marginal gains become small.

## Recurring failures

- **Frequency-based eviction (LFU / aging-LFU)** has been retried in multiple forms across cycles 1–3 — standard LFU, aggressive per-hit aging, and threshold-gated decay — and consistently underperforms or regresses. Workload-a's access pattern is recency-dominant; frequency counters introduce blind spots during item ramp-up and drain mildly-warm residents that recency ordering would have kept.
- **Ghost / victim-cache re-admission** has been tried in at least two cycles across multiple position configurations (MRU and Back-insert on re-admission) and consistently delivers noise-level or slightly negative movement. The stash eviction boundary is simply not where meaningful hit-rate is lost on this workload.
- **Non-MRU insertion positions** (probationary zones, LRU-entry for new/re-admitted items, demotion-at-LRU asymmetry) regressed in every test in cycle 5; withholding MRU status increases eviction of items that still have residual access heat.
- **Pushing hot-tier topology past the confirmed peak** (28/4 split) caused a regression in cycle 3; insufficient stash capacity prevents cold items from surviving long enough for re-entry.

## Open questions

- With topology, re-entry positioning, admission control, and ghost sets all explored or exhausted, what structurally new axes remain? Adaptive segment sizing, eviction cost weighting, and TTL-based decay have not been probed.
- Would frequency-based or scan-resistant eviction become genuinely competitive under a different access pattern — scan-heavy, burst-cold, or uniform traces — where recency ordering is less dominant?
- Does a capacity stress test (smaller total cache size) create conditions where probationary or frequency-gated schemes finally pay off, because marginal eviction quality matters more?
- Is there a proportional or access-count-weighted re-entry position more nuanced than full MRU that could extract additional signal from the position lever?

## Per-policy notes

- **LRU / MRU (position-based recency):** Dominant on workload-a across all cycles. MRU insertion at every boundary — initial insertion, promotion, and re-admission — consistently preserves the most hits. The confirmed optimal geometry (24/8 split, MRU re-entry) forms the current compound baseline.
- **LFU / aging-LFU (frequency-based):** Structurally incompatible with workload-a's access distribution. Aggressive decay drains warm items; threshold gating replaces one failure mode with another (slow counter ramp-up). Should not be revisited on this workload unless a different trace motivates it.
- **Segmented-LRU (hot/cold asymmetry):** Textbook demotion-at-LRU is contraindicated; items displaced from the hot tier retain residual heat and need MRU privilege in the destination tier to survive to their next access. The classical asymmetric design assumes a sharper hot/cold divide than workload-a exhibits.
- **Ghost sets:** Consistently noise-level on workload-a regardless of ghost set position or re-admission insertion point. Permanently retired as a lever for this workload.
- **Stash (probing tier):** Confirmed role is admission filtering, not just capacity buffering. Optimal allocation is 8 slots (25% of total); both under-allocation (4 slots) and over-allocation (16+ slots) hurt. Any future variant must preserve full stash traversal for new cold insertions.
