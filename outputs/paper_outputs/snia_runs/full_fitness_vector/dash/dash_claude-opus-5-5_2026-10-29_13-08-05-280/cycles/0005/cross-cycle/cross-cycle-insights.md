# Cross-Cycle Insights (cycles 0001–0005)

## Recurring wins

- **Gate who enters a tier on reuse evidence.** This is the most reliable hit-rate lever across cycles 1–4. It held for the stash reuse gate (cycles 1–2), the full-target occupancy guard (cycle 3) and the full-probing guard (cycle 4). Asking for one more hit before entering a *full* bucket cuts the cascades that evict reused residents.
- **Fewer multi-bucket moves improves both hit-rate and latency.** Promotions, demotion cascades and stash churn are the shared cost in every cycle (1, 3, 4, 5). Changes that cut these moves win on both dimensions. Changes that add them lose on both.
- **Remove work instead of deferring it.** Guards that eliminate moves made the tail faster (cycle 3). Guards that only postpone them made the get tail slower (cycle 4). Replacing the cascade with a direct swap gave the tail back with identical hit-rate (cycle 5). Where displaced entries go matters far less than when promotion fires.
- **Keep the common hit path lean.** Stripping the per-hit counter writes from the gate kept nearly all of its hit-rate and recovered median get (cycle 2).
- **Shrink Item by attacking the widest field.** Metadata-size was blocked by `timestamp: Instant`. Narrowing it to a `u32` tick plus a `u8` counter reached 24 bytes (cycle 4). With a per-segment tick there is no contention, and the smaller Item also made puts and the get tail faster (cycle 5).
- **Large caches are where hit-rate headroom lives.** Reuse-aware gains and losses showed up mostly in large-cache cells, especially hm_0.

## Recurring failures

- **Placement by free space alone** (balanced or free-slot placement, cycles 1–2). It does nothing under pressure and lets cold evictees fill free slots. Dropped.
- **Stacking "protect reused items" mechanisms on top of the gate.** MRU demotion landing and flag-limited free-slot placement duplicate evidence the gate already captures. They gave little or no hit-rate (cycle 2).
- **Making re-entry cheaper.** Re-admission credits (cycle 3) and faster re-promotion speed up swaps between residents and add churn. Hit-rate fell in every cell.
- **Bookkeeping on the common hit path.** Counter writes, compares and reset checks cost median get every time, even when they rarely change behaviour (cycles 1, 2, 5).
- **Narrowing a small field alone** (cycle 3 `u8` counter). It disappears into alignment padding until the widest field shrinks.
- **A global atomic tick.** A contended write on every insert erased the layout gain (cycle 4).
- **Fixed stash thresholds that ignore cache size.** They help large caches and hurt small ones, because a key in stash is evicted too quickly in a small cache to collect extra hits. Little headroom is left in tuning the threshold value.
- **Trusting latency-only Winners.** Several "wins" (cycles 2–5) moved latency on paths their diffs never touched.

## Open questions

- **How large is per-op-time noise?** A re-run of an identical baseline has been requested since cycle 2 and has never been done. This blocks conclusions about small latency changes.
- **Do separately-won changes combine?** In particular, does the 24-byte per-segment-tick Item add up with the direct-swap probing promotion?
- **Which paths now dominate get and put p99?** The full-probing arm appears to fire rarely, so stash→target moves and put eviction are the likely candidates. How often does each path actually fire?
- **Can the strict probing reset leave the hot path?** For example, a lazy tick-based staleness check on the rare hottest arm, keeping its web_0-large hit-rate gain without the median get cost.
- **Is the stash-side size gate still needed?** The target-side guard largely made it redundant. Should it be removed for simplicity?
- **What limits web_0 hit-rate at large size?** Free-slot placement does not explain it.
- **Can a cheap, write-free victim guard recover the last large-cache hits?** The victim comparison dropped in cycle 2 bought a small hm_0 gain.

## Per-policy notes

- **Dash segment promotion/placement (stash → target → probing):**
  - Insert position and promotion rules are coupled: where an item lands sets how quickly it climbs.
  - Door guards on full buckets work at both the stash→target and target→probing tiers.
  - A guard helps small caches only where the candidate stays long enough to collect hits. Target does; stash does not.
  - The current best line is: threshold-only stash gate, occupancy guard on full target, full-probing guard with direct swap, and possibly a size gate.
- **ClassicLRU (the evaluated policy for Item layout):**
  - It never reads the timestamp, so Item layout changes are hit-rate-neutral. Only the cost of maintaining the field matters.
  - A per-segment tick, or no tick at all, is preferred.
- **Trace notes:**
  - hm_0 responds most to reuse-aware admission at large size.
  - web_0 large keeps residual headroom that no placement change has unlocked yet.
