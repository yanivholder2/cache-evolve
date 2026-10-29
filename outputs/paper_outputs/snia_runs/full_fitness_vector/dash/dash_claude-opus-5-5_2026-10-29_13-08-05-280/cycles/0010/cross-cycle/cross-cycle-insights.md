## Recurring wins

- Admission control at the door is the strongest lever (0001–0005). Ask for extra reuse evidence before a key enters a *full* bucket. The two-hit stash gate, the full-target guard and the full-probing guard all raised hit-rate by cutting the cascades that evict reused residents.
- Cutting multi-bucket moves helps both hit-rate and latency, because promotions plus demotion cascades are a cost the two dimensions share. Removing work outright (dropping the counter writes, adding the occupancy guard, the single eviction) beats deferring it or making it lazy.
- The demoted probing resident is disposable (0007–0008, on two lineages). Evicting it outright costs no hit-rate and frees a slot. That beats both the cascade, which costs work, and the swap, which leaves buckets saturated.
- Bounded, aging stash-exit protection gained on hm_0 and lost in no cell (0009, confirmed in 0010). It needs a recency gate, a limit on how many entries are protected, and no MRU refresh.
- Item layout: `timestamp: Instant` was the only thing keeping Item from shrinking. A `u32` tick plus a `u8` counter gives 24 bytes. The tick must be per-segment, because a global atomic tick added contention that cancelled the gain.
- Levers on *different* paths stack (probing window plus stash protection, 0010). Levers on the *same* arm overlap and do not stack (0007).
- The simulator is deterministic: an identical-copy control matches to the last digit. So any hit-rate delta on a lineage is real, however small.

## Recurring failures

- Any bookkeeping on the common hit path (counter writes, compares, strict resets) shows up in median get. Lazy checks on rare arms move the cost into the tail but rarely remove it.
- Mechanisms that make it cheaper to come back, or "protect reused items" on top of the gate, add little or nothing, or hurt. This covers re-admission credits, MRU demotion landing and flag-limited free-slot placement, which duplicate what the gate already captures.
- Placement driven only by free space does not move hit-rate.
- Swaps in place of cascades never free slots. Buckets stay full and get/put tails rise. Swapping at the stash→target tier did not change hit-rate at all.
- `PROBE_WINDOW` tuning does not move hit-rate in either direction. Widening it costs get latency, and narrowing it showed no measurable latency gain.
- Unbounded stash-victim protection starves fresh arrivals and loses hit-rate everywhere, small caches most.
- web_0-large has stayed flat under placement, stash exit, the probing arm, window tuning and ghost re-admission at the stash gate.
- Cross-session latency comparisons: uniform shifts, including on untouched put paths, were environmental drift in 0005–0010. Most latency-only "Winners" before 0009 are unreliable. Even after the control rule, writers kept comparing against the cross-session parent instead of the same-session control.
- No variant writer could read overlay source, so every mechanism is inferred from the hypotheses.

## Open questions

- Is web_0-large capacity-bound? A single test with a longer-lived or larger off-Item ghost should settle it. If that stays flat, stop targeting the cell.
- Against the same-session control, are 0010 v003's put p99 rise (from the ghost scan on stash insert) and v002's put jump real?
- Does the ghost credit stack with the narrower window? They sit on different paths, and both nudged hm_0 up.
- How often do the guarded full-target and full-probing arms actually fire, and which paths now dominate get and put p99?
- Could thresholds or windows scaled to cache or stash size recover the remaining small-cache hm_0 losses?
- Would tuning `STASH_PROTECT_AGE` give more hm_0 gain?

## Per-policy notes

- ClassicLRU (the evaluated policy) never reads the per-Item timestamp. Tick narrowing and per-segment ticks therefore cannot change its hit-rate and only affect maintenance cost. A tick that is removed entirely for ClassicLRU is still worth considering.
- The stash's LRU-position eviction is already scan-resistant. Only aging, bounded reprieves improve on it. Protection that never ages out does worse.
- Gains are trace-dependent. hm_0 rewards short-range reuse (gate, stash protection, ghost). web_0, especially large, has resisted every lever. Large caches are where admission-guard gains and losses appear. Small caches are hurt by stash-side guards because keys stay in stash too briefly to collect extra hits.
