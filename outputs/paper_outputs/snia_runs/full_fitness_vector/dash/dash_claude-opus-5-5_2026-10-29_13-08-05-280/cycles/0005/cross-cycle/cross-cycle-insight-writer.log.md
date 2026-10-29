# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-29T13:51:36+00:00

## timeline

### 13:51:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0005)\n\n## Recurring wins\n\n- **Gate who enters a tier on reuse evidence.** This is the most reliable hit-rate lever across cycles 1–4. It held for the stash reuse gate (cycles 1–2), the full-target occupancy guard (cycle 3) and the full-probing guard (cycle 4). Asking for one more hit before entering a *full* bucket cuts the cascades that evict reused residents.\n- **Fewer multi-bucket moves improves both hit-rate and latency.** Promotions, demotion cascades and stash churn are the shared cost in every cycle (1, 3, 4, 5). Changes that cut these moves win on both dimensions. Changes that add them lose on both.\n- **Remove work instead of deferring it.** Guards that eliminate moves made the tail faster (cycle 3). Guards that only postpone them made the get tail slower (cycle 4). Replacing the cascade with a direct swap gave the tail back with identical hit-rate (cycle 5). Where displaced entries go matters far less than when promotion fires.\n- **Keep the common hit path lean.** Stripping the per-hit counter writes from the gate kept nearly all of its hit-rate and recovered median get (cycle 2).\n- **Shrink Item by attacking the widest field.** Metadata-size was blocked by `timestamp: Instant`. Narrowing it to a `u32` tick plus a `u8` counter reached 24 bytes (cycle 4). With a per-segment tick there is no contention, and the smaller Item also made puts and the get tail faster (cycle 5).\n- **Large caches are where hit-rate headroom lives.** Reuse-aware gains and losses showed up mostly in large-cache cells, especially hm_0.\n\n## Recurring failures\n\n- **Placement by free space alone** (balanced or free-slot placement, cycles 1–2). It does nothing under pressure and lets cold evictees fill free slots. Dropped.\n- **Stacking \"protect reused items\" mechanisms on top of the gate.** MRU demotion landing and flag-limited free-slot placement duplicate evidence the gate already captures. They gave little or no hit-rate (cycle 2).\n- **Making re-entry cheaper.** Re-admission credits (cycle 3) and faster re-promotion speed up swaps between residents and add churn. Hit-rate fell in every cell.\n- **Bookkeeping on the common hit path.** Counter writes, compares and reset checks cost median get every time, even when they rarely change behaviour (cycles 1, 2, 5).\n- **Narrowing a small field alone** (cycle 3 `u8` counter). It disappears into alignment padding until the widest field shrinks.\n- **A global atomic tick.** A contended write on every insert erased the layout gain (cycle 4).\n- **Fixed stash thresholds that ignore cache size.** They help large caches and hurt small ones, because a key in stash is evicted too quickly in a small cache to collect extra hits. Little headroom is left in tuning the threshold value.\n- **Trusting latency-only Winners.** Several \"wins\" (cycles 2–5) moved latency on paths their diffs never touched.\n\n## Open questions\n\n- **How large is per-op-time noise?** A re-run of an identical baseline has been requested since cycle 2 and has never been done. This blocks conclusions about small latency changes.\n- **Do separately-won changes combine?** In particular, does the 24-byte per-segment-tick Item add up with the direct-swap probing promotion?\n- **Which paths now dominate get and put p99?** The full-probing arm appears to fire rarely, so stash→target moves and put eviction are the likely candidates. How often does each path actually fire?\n- **Can the strict probing reset leave the hot path?** For example, a lazy tick-based staleness check on the rare hottest arm, keeping its web_0-large hit-rate gain without the median get cost.\n- **Is the stash-side size gate still needed?** The target-side guard largely made it redundant. Should it be removed for simplicity?\n- **What limits web_0 hit-rate at large size?** Free-slot placement does not explain it.\n- **Can a cheap, write-free victim guard recover the last large-cache hits?** The victim comparison dropped in cycle 2 bought a small hm_0 gain.\n\n## Per-policy notes\n\n- **Dash segment promotion/placement (stash → target → probing):**\n  - Insert position and promotion rules are coupled: where an item lands sets how quickly it climbs.\n  - Door guards on full buckets work at both the stash→target and target→probing tiers.\n  - A guard helps small caches only where the candidate stays long enough to collect hits. Target does; stash does not.\n  - The current best line is: threshold-only stash gate, occupancy guard on full target, full-probing guard with direct swap, and possibly a size gate.\n- **ClassicLRU (the evaluated policy for Item layout):**\n  - It never reads the timestamp, so Item layout changes are hit-rate-neutral. Only the cost of maintaining the field matters.\n  - A per-segment tick, or no tick at all, is preferred.\n- **Trace notes:**\n  - hm_0 responds most to reuse-aware admission at large size.\n  - web_0 large keeps residual headroom that no placement change has unlocked yet.\n"
}
```

### 13:52:03 chat

I wrote `cross-cycle-insights.md`, built only from the ledger for cycles 0001–0005, with the four required sections in order.

**Biggest open gap:** the per-op-time noise has still never been measured. Every cycle since 0002 asked for a re-run of an identical baseline, and none has been done. Until it is, small latency-only "wins" can't be trusted.

- **Recurring wins:**
  - Requiring reuse evidence before an item enters a tier, especially a full bucket, is the steadiest hit-rate lever. It held at both the stash→target and target→probing tiers.
  - Changes that cut multi-bucket moves improve hit-rate and latency together.
  - Removing work beats postponing it, as the direct swap showed.
  - The common hit path should stay free of extra checks and bookkeeping.
  - Item reached 24 bytes once `timestamp` shrank, and with a per-segment tick the smaller layout also sped up puts and the get tail.
- **Recurring failures:**
  - Placing items by free space alone.
  - Adding more "protect reused items" mechanisms on top of the gate.
  - Making it cheaper for items to come back after demotion.
  - Bookkeeping on the common hit path.
  - Narrowing a small field before the widest one.
  - A global atomic tick.
  - Stash thresholds that ignore cache size.
- **Open questions:**
  - Do the 24-byte Item and the direct swap combine cleanly?
  - Which paths now drive the get and put tails?
  - Can the strict probing reset move off the common hit path?
  - Should the now-mostly-redundant stash size gate be removed?
  - What still limits web_0 at the large size?
  - Can a cheap victim guard recover the last large-cache hits?
- **Per-policy notes:**
  - **Dash promotion and placement:** where an item lands is tied to how quickly it gets promoted. The note also lists the current best combination of guards.
  - **ClassicLRU:** layout changes don't affect hit-rate because it never reads the timestamp. Only the cost of keeping the field matters.
  - **Traces:** hm_0 and web_0 behave differently at the large size, and each gets its own note.


---

- finished: 2026-10-29T13:52:03+00:00
- duration: 26.50s
- status: ok
- input_tokens: 4
- output_tokens: 2672
- cache_read_tokens: 39360
- cache_write_tokens: 11357
- cost_usd: 0.152184
