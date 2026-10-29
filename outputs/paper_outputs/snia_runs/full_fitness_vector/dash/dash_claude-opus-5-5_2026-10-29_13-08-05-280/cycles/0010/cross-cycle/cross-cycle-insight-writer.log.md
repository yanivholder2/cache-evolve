# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-29T14:39:14+00:00

## timeline

### 14:39:21 thinking



### 14:39:36 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Admission control at the door is the strongest lever (0001–0005). Ask for extra reuse evidence before a key enters a *full* bucket. The two-hit stash gate, the full-target guard and the full-probing guard all raised hit-rate by cutting the cascades that evict reused residents.\n- Cutting multi-bucket moves helps both hit-rate and latency, because promotions plus demotion cascades are a cost the two dimensions share. Removing work outright (dropping the counter writes, adding the occupancy guard, the single eviction) beats deferring it or making it lazy.\n- The demoted probing resident is disposable (0007–0008, on two lineages). Evicting it outright costs no hit-rate and frees a slot. That beats both the cascade, which costs work, and the swap, which leaves buckets saturated.\n- Bounded, aging stash-exit protection gained on hm_0 and lost in no cell (0009, confirmed in 0010). It needs a recency gate, a limit on how many entries are protected, and no MRU refresh.\n- Item layout: `timestamp: Instant` was the only thing keeping Item from shrinking. A `u32` tick plus a `u8` counter gives 24 bytes. The tick must be per-segment, because a global atomic tick added contention that cancelled the gain.\n- Levers on *different* paths stack (probing window plus stash protection, 0010). Levers on the *same* arm overlap and do not stack (0007).\n- The simulator is deterministic: an identical-copy control matches to the last digit. So any hit-rate delta on a lineage is real, however small.\n\n## Recurring failures\n\n- Any bookkeeping on the common hit path (counter writes, compares, strict resets) shows up in median get. Lazy checks on rare arms move the cost into the tail but rarely remove it.\n- Mechanisms that make it cheaper to come back, or \"protect reused items\" on top of the gate, add little or nothing, or hurt. This covers re-admission credits, MRU demotion landing and flag-limited free-slot placement, which duplicate what the gate already captures.\n- Placement driven only by free space does not move hit-rate.\n- Swaps in place of cascades never free slots. Buckets stay full and get/put tails rise. Swapping at the stash→target tier did not change hit-rate at all.\n- `PROBE_WINDOW` tuning does not move hit-rate in either direction. Widening it costs get latency, and narrowing it showed no measurable latency gain.\n- Unbounded stash-victim protection starves fresh arrivals and loses hit-rate everywhere, small caches most.\n- web_0-large has stayed flat under placement, stash exit, the probing arm, window tuning and ghost re-admission at the stash gate.\n- Cross-session latency comparisons: uniform shifts, including on untouched put paths, were environmental drift in 0005–0010. Most latency-only \"Winners\" before 0009 are unreliable. Even after the control rule, writers kept comparing against the cross-session parent instead of the same-session control.\n- No variant writer could read overlay source, so every mechanism is inferred from the hypotheses.\n\n## Open questions\n\n- Is web_0-large capacity-bound? A single test with a longer-lived or larger off-Item ghost should settle it. If that stays flat, stop targeting the cell.\n- Against the same-session control, are 0010 v003's put p99 rise (from the ghost scan on stash insert) and v002's put jump real?\n- Does the ghost credit stack with the narrower window? They sit on different paths, and both nudged hm_0 up.\n- How often do the guarded full-target and full-probing arms actually fire, and which paths now dominate get and put p99?\n- Could thresholds or windows scaled to cache or stash size recover the remaining small-cache hm_0 losses?\n- Would tuning `STASH_PROTECT_AGE` give more hm_0 gain?\n\n## Per-policy notes\n\n- ClassicLRU (the evaluated policy) never reads the per-Item timestamp. Tick narrowing and per-segment ticks therefore cannot change its hit-rate and only affect maintenance cost. A tick that is removed entirely for ClassicLRU is still worth considering.\n- The stash's LRU-position eviction is already scan-resistant. Only aging, bounded reprieves improve on it. Protection that never ages out does worse.\n- Gains are trace-dependent. hm_0 rewards short-range reuse (gate, stash protection, ghost). web_0, especially large, has resisted every lever. Large caches are where admission-guard gains and losses appear. Small caches are hurt by stash-side guards because keys stay in stash too briefly to collect extra hits.\n"
}
```

### 14:39:44 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle directory. It's built only from the ledger digest for cycles 0001–0010 and uses the four required sections in order.

- **Recurring wins:**
  - The strongest lever is asking for extra reuse evidence before a key enters a full bucket.
  - Removing work outright beats delaying it or making the check lazy.
  - The demoted probing resident can simply be evicted with no hit-rate loss.
  - Stash protection that ages out and is limited in size helps hm_0.
  - Item shrinks to 24 bytes with a per-segment tick.
  - Changes on different paths add up; changes on the same promotion arm don't.
  - The simulator is deterministic, so even small hit-rate changes are real.
- **Recurring failures:**
  - Any check on the common hit path raises median get latency.
  - Extra "protect reused items" mechanisms on top of the gate add little or nothing.
  - Swaps keep buckets full and push up get and put tails.
  - Tuning `PROBE_WINDOW` doesn't change hit-rate.
  - Stash protection that never ages out loses everywhere.
  - web_0-large hasn't moved under any lever tried.
  - Latency compared across sessions was drift, so most latency-only Winners before cycle 0009 can't be trusted. Writers also kept ignoring the same-session control.
- **Open questions:**
  - Whether web_0-large is simply too small to hold its working set (one longer-lived ghost test, then stop).
  - Re-checking cycle 0010's put-latency moves against the same-session control.
  - Whether the ghost credit and the narrower window add up.
  - How often the guarded arms actually fire.
  - Size-scaled thresholds for small caches.
  - Tuning `STASH_PROTECT_AGE`.
- **Per-policy notes:** ClassicLRU never reads the timestamp, so tick changes only affect upkeep cost, not hit-rate. The stash's existing LRU-order eviction already resists scans. Results depend on the trace: hm_0 rewards short-range reuse, web_0 hasn't responded to anything.


---

- finished: 2026-10-29T14:39:45+00:00
- duration: 31.07s
- status: ok
- input_tokens: 4
- output_tokens: 2531
- cache_read_tokens: 44824
- cache_write_tokens: 16686
- cost_usd: 0.193089
