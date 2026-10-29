# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-29T14:10:26+00:00

## timeline

### 14:10:52 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **Gating who enters a full bucket is the strongest lever.** Asking for one more piece of reuse evidence before a promotion into a *full* tier helped at each tier where it was tried: the stash reuse gate (0001–0002), the stash→full-target guard (0003) and the full-probing guard (0004). Each one cuts cascades that would push out reused residents.\n- **Fewer multi-bucket moves improves hit-rate and latency together.** Stash churn and cascade length are the shared cost behind both dimensions (0001, 0003). Changes that *remove* moves gave clean wins. Changes that only *defer* moves raised hit-rate but hurt the tail (0004, 0005).\n- **Keep the common hit path lean.** Removing per-hit counter writes gave a faster get path (0002). Every later check on the hottest/target hit path (0002 v003, 0004 v003, 0005 v003) showed up as higher median get.\n- **Put rare checks on rare arms, and use them as throttles.** A lazy tick-age check on the hottest/full arm kept p50 flat (0006). Used as a limit on how often the swap fires, it also brought the tail down (0007 v003).\n- **Item layout: shrink `timestamp`, not the counter.** Item size was set by `Instant` (16 bytes, align 8). With a `u32` tick plus a `u8` counter, Item fits in 24 bytes (0004). A per-segment tick removes the contention from a global atomic and makes the smaller layout faster too (0005).\n- **The guard's evictee can be dropped.** Evicting the full-probing evictee outright cost no hit-rate in any cell (0007 v002). Cascade re-homing was not keeping useful keys.\n\n## Recurring failures\n\n- **Re-promotion shortcuts.** MRU landing (0001), reuse-aware demotion landing (0002), and the re-admission credit (0003) were redundant with the gate or added churn. The credit lowered hit-rate in every cell.\n- **Placing items by free space alone** (balanced/free-slot placement, 0001–0002) never moved hit-rate. It also let cold evictees fill slack.\n- **Assuming promotion levers stack.** Size gate, recency window, swap and cascade all fix the same cost: promoting into a slot and evicting a resident that is still useful. Once one lever is present, the others add almost nothing (0005–0007). Whether a lever is \"redundant\" depends on the lineage. The size gate is redundant under the swap, but under the cascade it protects hm_0 in small caches.\n- **Swaps as a tail fix.** A swap never frees a slot, so buckets stay saturated and more ops take the slow full-bucket paths. It helped the tail once (0005) but hurt both tails on the merged parent (0006). The stash→target swap left hit-rate unchanged and slowed latency (0006 v003).\n- **Fixed thresholds across cache sizes.** In small caches, keys leave the stash before they can collect an extra hit, so stash-side guards hurt small caches unless they are size-gated (0003–0004). Target-side guards do not have this problem.\n- **Latency-only Winners.** Cycles 0003–0007 each show at least one uniform latency shift, often on ops the diff never touches. Most latency-only verdicts are probably noise.\n\n## Open questions\n\n- **Noise baseline (top priority, requested in cycles 0002–0007 and never run).** Re-run an identical baseline, and 0007 v002 next to its parent, on the same host, to size per-op-time variance.\n- Does replacing the swap with a single outright eviction on the window-throttled lineage keep the swap's hit-rate, free slots, and cost less than both the swap and the cascade?\n- How often do the guarded full-probing and full-target arms actually fire? Which paths dominate get and put p99 now: stash→target, put eviction, or something else?\n- Would a cache-sized `PROBE_WINDOW` (wider in small caches) win back the hm_0-small loss?\n- Can the non-hottest-hit staleness signal (0005 v003's large-cache gain) be captured without touching the common hit path?\n- Does a cheap write-free victim guard recover the large-cache hits lost when the victim comparison was dropped (0002)?\n- What limits web_0 at the large cache size, given that placement changes do not affect it?\n\n## Per-policy notes\n\n- **Dash segment (stash / target / probing tiers):** every cycle's changes were here. The current lineage is: threshold-only stash gate → occupancy guard on full targets → full-probing guard (two hottest hits) → swap or cascade on that arm, throttled by a tick-age window (0007 v003 baseline). Small-cache size gate: drop it on the swap lineage, keep it on the cascade lineage.\n- **ClassicLRU (evaluated policy):** never reads `timestamp`, so tick or layout changes cannot affect hit-rate. The only cost is maintaining the tick. A per-segment tick, or no tick at all, is preferred.\n- **Traces:** reuse-aware gains are largest on hm_0 and at the large cache size. web_0 large is the remaining hit-rate frontier. hm_0 small is the cell most sensitive to removing gates or windows.\n- **Caveat:** no variant insight could read overlay source, so the mechanisms above are inferred from the hypotheses, not confirmed from the code.\n"
}
```

### 14:10:59 chat

I've written `cross-cycle-insights.md` for cycles 0001–0007, using the four required sections in order. I built it only from the ledger digest and didn't open the previous version.

- **Recurring wins:** The strongest lever is requiring one extra sign of reuse before an item can move into a full bucket. It worked at every tier where it was tried. Changes that remove multi-bucket moves improve hit-rate and latency together, while changes that only postpone those moves raise hit-rate but slow the worst-case latency. Adding checks to the common hit path consistently made gets slower. Cheap checks on the rare branch worked best as a limit on how often a swap fires. Item metadata finally shrank to 24 bytes once `timestamp` was narrowed.
- **Recurring failures:** Making it faster for evicted items to climb back up never paid off, and neither did placing items by free space alone. Promotion changes on the same path overlap rather than add up. Swaps keep buckets full, which slows later operations. Stash-side thresholds hurt small caches. Most improvements that showed up only in latency were probably measurement noise.
- **Open questions:** The top item is a re-run of an identical baseline to measure how noisy the timing benchmark is. Every insight from cycle 0002 on has asked for it, and it has never been done. Next is testing a single outright eviction in place of the swap on the current baseline.
- **Per-policy notes:** These cover the Dash segment's current line of changes, ClassicLRU (which never reads `timestamp`), and which traces and cache sizes behave differently.

None of the per-cycle reports could read the variants' source code, so every mechanism in the file is inferred from the hypotheses. The file says so in its last bullet.


---

- finished: 2026-10-29T14:11:00+00:00
- duration: 34.11s
- status: ok
- input_tokens: 4
- output_tokens: 2533
- cache_read_tokens: 41433
- cache_write_tokens: 13463
- cost_usd: 0.166667
