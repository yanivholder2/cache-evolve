# Cross-Cycle Insights

## Recurring wins

- **Admission guards on "who enters" a bucket are the most dependable hit-rate lever.** This holds across cycles 1, 3, 4 and 5. Examples: the two-hit stash gate, the extra hit needed to enter a full target, and the two hottest hits needed to enter a full probing bucket. Each one asks for one more piece of reuse evidence before a full bucket is entered, which cuts the cascades that push reused residents out.
- **Removing work beats deferring it.** Fewer multi-bucket moves (promotions plus demotion cascades, stash churn) improve hit-rate and latency together. When a change dropped the moves outright, the tail got faster (cycles 3, 5). When it only postponed them, the tail did not improve (cycle 4).
- **The displaced probing resident is disposable.** Evicting it outright, instead of re-homing it by cascade or swap, cost no hit-rate on two lineages (cycles 7, 8) and slightly helped large-cache hm_0. It also frees a slot, which avoids the occupancy cost of a swap.
- **Using a recency window as a throttle** on how often the costly swap fires lowered the tail (cycle 7). It is the only lazy check that actually reduced cost rather than moving it.
- **Shrinking Item layout worked once the real blocker was removed.** `timestamp: Instant` was the field holding size up. Replacing it with a `u32` tick, plus a `u8` counter, gives a 24-byte Item. The tick has to be per-segment: a global atomic caused contention regressions (cycles 4, 5).
- **Large cache sizes, and hm_0 in particular, are where reuse-aware changes win or lose most of their hit-rate.**

## Recurring failures

- **Any bookkeeping on the common target-hit path raises median get.** This includes counter writes, compares, and resets. It has shown up in at least four cycles (1, 2, 5, 8), even when the logic almost never fires.
- **Stacking promotion levers on the same arm doesn't add up.** Pairs such as swap with gate removal, or window with cascade, address the same cost. Once one is in place, the other adds nothing (cycle 7).
- **"Faster comeback" mechanisms lose hit-rate.** Re-admission credits, MRU demotion landing and wider probe windows all raise churn or cost without gaining hits (cycles 2, 3, 8).
- **Placement based only on capacity does nothing for hit-rate** (cycles 1, 2). This covers free-slot balanced placement and its flag-limited form.
- **Swaps that never free a slot keep buckets saturated** and push more operations onto slow full-bucket paths. The tail cost often outweighs the hit-rate gain (cycle 6).
- **Protecting stash entries with no aging** (preferring to evict never-hit entries) starves fresh arrivals and loses hit-rate everywhere, small caches most (cycle 8). The stash's LRU-position exit is already the right choice.
- **Size-gated thresholds have little headroom left.** The stash size gate is redundant on the swap/eviction lineage, though it was still holding up small-cache hm_0 under the cascade.
- **Latency-only "Winners" can't be trusted.** Cycles 3 and 5 through 8 each showed uniform shifts that touched paths the diff never changed, including put. An identical-baseline noise re-run has been requested for seven cycles and has never been done.

## Open questions

- **How large is per-op-time noise?** This blocks everything else. Run the parent, an identical copy of it, and recent candidates together on one host in one session, as a hypothesis of its own.
- **How often do the guarded full-probing and full-target arms actually fire?** And which paths dominate get and put p99 now: stash→target moves or put eviction?
- **What limits hit-rate in hm_0-small and web_0-large on the current lineage?** Throttle tuning and stash-exit protection have been ruled out. The remaining candidates are the arm's mechanics and admission.
- **Can a non-hottest-hit staleness signal be captured without touching the common hit path?**
- **Would an aging bound make stash-exit protection viable,** or is that lever dead?
- **Does narrowing the window on the eviction lineage cut latency** without costing hit-rate? Would a window sized to the cache help small caches?

## Per-policy notes

- **ClassicLRU (the only evaluated policy):**
  - It never reads the timestamp, so changing the tick or layout cannot change its hit-rate. Only the cost of maintaining the field matters.
  - Removing the tick entirely may be an option.
- **Dash segment tiers (target / probing / stash):**
  - The current baseline lineage is the per-segment tick with the 24-byte Item, the full-target door guard, the full-probing two-hit guard, a window throttle, and a single outright eviction of the probing resident.
  - Guards help small caches only at tiers where candidates stay long enough to collect hits. Target qualifies; stash does not, unless the cache is large.
