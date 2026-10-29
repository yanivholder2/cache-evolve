## Recurring wins

- **Admission ("who enters") is the strongest lever.** It held up in cycles 0001–0004. The two-hit stash gate gave most of the early hit-rate gains. Door guards that ask for one extra piece of reuse evidence before entering a *full* bucket worked at both the stash→target and target→probing tiers. Guards help small caches only if the candidate stays put long enough to collect the extra hit: target residents do, stash residents do not.
- **The count of multi-bucket moves drives both hit-rate and latency.** Cutting promotions, cascades and stash churn tends to help both dimensions together (0001, 0003). Removing work beats deferring it (0004, 0005).
- **Displaced residents are disposable.** Evicting the probing evictee outright cost no hit-rate on the cascade lineage (0007) or the window lineage (0008). It frees a slot without the cascade's multi-bucket work. Eviction now beats both swap and cascade.
- **Keep the common hit path free of bookkeeping.** Counter writes, resets and compares on the common target-hit path showed up in median get every time they were tried (0002, 0005, 0006). Put checks only on the rare arm.
- **Item layout:** `timestamp: Instant` was the only thing blocking metadata-size. Replacing it with a per-segment `u32` tick plus a `u8` counter gave 24 bytes, and the smaller Item also helped the slow-path latency (0004, 0005). A global atomic tick caused contention.
- **Stash-exit protection works only when it is bounded and ages out.** Recency-gated protection with a single reprieve and at most two slots gained on hm_0 and lost nowhere (0009).
- **Identical-copy controls are worth their slot.** The 0009 control showed that cross-session latency drifts uniformly. Comparing ops within one run (put tail vs get tail) still gave usable signal.

## Recurring failures

- **Faster re-promotion and re-admission credits** sped up swaps among reused residents, added churn, and lost hit-rate everywhere (0003).
- **Placement based on free space alone** (balanced or free-slot placement, flagged free slots) never moved hit-rate (0001, 0002).
- **Stacking promotion levers on the same arm:** they overlap rather than add, because each fixes the same cost of evicting a still-useful resident (0002, 0007). Each extra reuse mechanism after the gate mostly duplicated it.
- **Swaps:** they keep buckets saturated and push ops onto slow full-bucket paths. Their tail results flip sign depending on the lineage (0005 vs 0006), and swapping between stash and target did nothing (0006).
- **`PROBE_WINDOW` as a hit-rate knob:** it made no hit-rate difference whether widened or narrowed. Widening raised get p50 and p99 (0008, 0009). Treat it purely as a latency throttle.
- **Unbounded stash-victim protection** filled the stash with one-hit keys and starved fresh arrivals, and the losses were largest in small caches (0008).
- **Fixed thresholds that are not size-aware** trade large-cache gains for small-cache losses (0003). Size gates help only on some lineages: the stash size gate was redundant on the swap lineage and load-bearing under the cascade.
- **Latency-only Winners scored across sessions:** five cycles (0003–0008) had uniform shifts on untouched paths. 0009 confirmed these were environmental drift. Those verdicts should be discounted.
- Variant insights never had overlay source, so every mechanism claim so far is inferred from the hypotheses.

## Open questions

- **web_0-large has stayed flat** under stash exit, the window, arm eviction and free-slot placement. Is admission (gate or door guard thresholds) the remaining lever, or is the trace capacity-bound?
- Does 0009's bounded stash protection stack with a narrower `PROBE_WINDOW`? They act on different paths, so they might add, unlike promotion levers on the same arm.
- In the same session, is the eviction lineage actually faster than the swap lineage? Does a narrow window lower get latency?
- How often do the guarded full-probing and full-target arms fire? Which paths now dominate the get and put p99?
- What holds back hm_0-small on the window lineage? Could a window or threshold sized to the cache recover it?
- Is `STASH_PROTECT_AGE` tunable for more hm_0 gain without small-cache cost?

## Per-policy notes

- **ClassicLRU (the evaluated policy):** it never reads the timestamp, so changes to the tick or timestamp layout are hit-rate-neutral and only affect cost. The stash's LRU-position eviction is already scan-resistant, and only aged, bounded protection improved on it.
- **Dash tiers:** target→probing promotion is set by hottest-position hits, so insert position and promotion speed are coupled. Since 0004, the full-probing door guard is the hit-rate backbone. The current line is window throttle, single eviction, and bounded stash protection (0009 v003).
- **Traces:** hm_0 responds to reuse-aware and short-range-reuse levers at both sizes. web_0 responds weakly, and its gains appear mainly at the large size. Large caches are where the remaining hit-rate headroom has been.
- **Measurement rule going forward:** every cohort includes an identical-copy control of its parent, and latency is scored only against that control.
