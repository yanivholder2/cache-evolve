# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-29T14:29:55+00:00

## timeline

### 14:29:57 thinking



### 14:30:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Admission (\"who enters\") is the strongest lever.** It held up in cycles 0001–0004. The two-hit stash gate gave most of the early hit-rate gains. Door guards that ask for one extra piece of reuse evidence before entering a *full* bucket worked at both the stash→target and target→probing tiers. Guards help small caches only if the candidate stays put long enough to collect the extra hit: target residents do, stash residents do not.\n- **The count of multi-bucket moves drives both hit-rate and latency.** Cutting promotions, cascades and stash churn tends to help both dimensions together (0001, 0003). Removing work beats deferring it (0004, 0005).\n- **Displaced residents are disposable.** Evicting the probing evictee outright cost no hit-rate on the cascade lineage (0007) or the window lineage (0008). It frees a slot without the cascade's multi-bucket work. Eviction now beats both swap and cascade.\n- **Keep the common hit path free of bookkeeping.** Counter writes, resets and compares on the common target-hit path showed up in median get every time they were tried (0002, 0005, 0006). Put checks only on the rare arm.\n- **Item layout:** `timestamp: Instant` was the only thing blocking metadata-size. Replacing it with a per-segment `u32` tick plus a `u8` counter gave 24 bytes, and the smaller Item also helped the slow-path latency (0004, 0005). A global atomic tick caused contention.\n- **Stash-exit protection works only when it is bounded and ages out.** Recency-gated protection with a single reprieve and at most two slots gained on hm_0 and lost nowhere (0009).\n- **Identical-copy controls are worth their slot.** The 0009 control showed that cross-session latency drifts uniformly. Comparing ops within one run (put tail vs get tail) still gave usable signal.\n\n## Recurring failures\n\n- **Faster re-promotion and re-admission credits** sped up swaps among reused residents, added churn, and lost hit-rate everywhere (0003).\n- **Placement based on free space alone** (balanced or free-slot placement, flagged free slots) never moved hit-rate (0001, 0002).\n- **Stacking promotion levers on the same arm:** they overlap rather than add, because each fixes the same cost of evicting a still-useful resident (0002, 0007). Each extra reuse mechanism after the gate mostly duplicated it.\n- **Swaps:** they keep buckets saturated and push ops onto slow full-bucket paths. Their tail results flip sign depending on the lineage (0005 vs 0006), and swapping between stash and target did nothing (0006).\n- **`PROBE_WINDOW` as a hit-rate knob:** it made no hit-rate difference whether widened or narrowed. Widening raised get p50 and p99 (0008, 0009). Treat it purely as a latency throttle.\n- **Unbounded stash-victim protection** filled the stash with one-hit keys and starved fresh arrivals, and the losses were largest in small caches (0008).\n- **Fixed thresholds that are not size-aware** trade large-cache gains for small-cache losses (0003). Size gates help only on some lineages: the stash size gate was redundant on the swap lineage and load-bearing under the cascade.\n- **Latency-only Winners scored across sessions:** five cycles (0003–0008) had uniform shifts on untouched paths. 0009 confirmed these were environmental drift. Those verdicts should be discounted.\n- Variant insights never had overlay source, so every mechanism claim so far is inferred from the hypotheses.\n\n## Open questions\n\n- **web_0-large has stayed flat** under stash exit, the window, arm eviction and free-slot placement. Is admission (gate or door guard thresholds) the remaining lever, or is the trace capacity-bound?\n- Does 0009's bounded stash protection stack with a narrower `PROBE_WINDOW`? They act on different paths, so they might add, unlike promotion levers on the same arm.\n- In the same session, is the eviction lineage actually faster than the swap lineage? Does a narrow window lower get latency?\n- How often do the guarded full-probing and full-target arms fire? Which paths now dominate the get and put p99?\n- What holds back hm_0-small on the window lineage? Could a window or threshold sized to the cache recover it?\n- Is `STASH_PROTECT_AGE` tunable for more hm_0 gain without small-cache cost?\n\n## Per-policy notes\n\n- **ClassicLRU (the evaluated policy):** it never reads the timestamp, so changes to the tick or timestamp layout are hit-rate-neutral and only affect cost. The stash's LRU-position eviction is already scan-resistant, and only aged, bounded protection improved on it.\n- **Dash tiers:** target→probing promotion is set by hottest-position hits, so insert position and promotion speed are coupled. Since 0004, the full-probing door guard is the hit-rate backbone. The current line is window throttle, single eviction, and bounded stash protection (0009 v003).\n- **Traces:** hm_0 responds to reuse-aware and short-range-reuse levers at both sizes. web_0 responds weakly, and its gains appear mainly at the large size. Large caches are where the remaining hit-rate headroom has been.\n- **Measurement rule going forward:** every cohort includes an identical-copy control of its parent, and latency is scored only against that control.\n"
}
```

### 14:30:23 chat

I've written `cross-cycle-insights.md` to the cycle 0009 cross-cycle output folder, with the four required sections in order. It draws only on the ledger in your prompt. The previous insights file was not read.

- **Recurring wins:**
  - Controlling which items get into a full bucket ("who enters") is the strongest lever across cycles.
  - Doing fewer multi-bucket moves helps both hit-rate and latency.
  - The resident pushed out of probing can simply be evicted, at no hit-rate cost.
  - The common hit path should carry no bookkeeping.
  - Replacing `timestamp` with a per-segment `u32` tick got Item down to 24 bytes.
  - Protecting stash entries from eviction works only when the protection is limited and ages out.
  - Identical-copy controls are worth running, since they exposed the latency drift.
- **Recurring failures:**
  - Faster re-promotion and re-admission credits lost hit-rate.
  - Placing items based only on free space never moved hit-rate.
  - Stacking two promotion levers on the same arm doesn't add up; they overlap.
  - Swaps keep buckets full and slow the tail.
  - `PROBE_WINDOW` makes no hit-rate difference.
  - Unlimited stash protection hurt, worst in small caches.
  - Fixed thresholds that ignore cache size trade large-cache gains for small-cache losses.
  - Latency "wins" scored across sessions turned out to be environmental drift and should be discounted.
- **Open questions:**
  - What, if anything, can move web_0-large?
  - Does stash protection stack with a narrower window?
  - Is the eviction lineage actually faster when timed in the same session?
  - How often do the guarded arms actually fire?
  - What is holding back hm_0-small?
  - Can `STASH_PROTECT_AGE` be tuned for more hm_0 gain?
- **Per-policy notes:**
  - ClassicLRU never reads the timestamp, so layout changes only affect cost.
  - On the Dash side, I note the guard that has carried hit-rate since cycle 0004 and the current baseline (cycle 0009 v003).
  - hm_0 and web_0 respond differently to the levers tried.
  - From now on, latency is scored only against a same-session control.


---

- finished: 2026-10-29T14:30:23+00:00
- duration: 28.50s
- status: ok
- input_tokens: 4
- output_tokens: 2823
- cache_read_tokens: 43735
- cache_write_tokens: 15845
- cost_usd: 0.191983
