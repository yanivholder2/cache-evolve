# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Gating who enters a full bucket is the strongest lever.** Asking for one more piece of reuse evidence before a promotion into a *full* tier helped at each tier where it was tried: the stash reuse gate (0001–0002), the stash→full-target guard (0003) and the full-probing guard (0004). Each one cuts cascades that would push out reused residents.
- **Fewer multi-bucket moves improves hit-rate and latency together.** Stash churn and cascade length are the shared cost behind both dimensions (0001, 0003). Changes that *remove* moves gave clean wins. Changes that only *defer* moves raised hit-rate but hurt the tail (0004, 0005).
- **Keep the common hit path lean.** Removing per-hit counter writes gave a faster get path (0002). Every later check on the hottest/target hit path (0002 v003, 0004 v003, 0005 v003) showed up as higher median get.
- **Put rare checks on rare arms, and use them as throttles.** A lazy tick-age check on the hottest/full arm kept p50 flat (0006). Used as a limit on how often the swap fires, it also brought the tail down (0007 v003).
- **Item layout: shrink `timestamp`, not the counter.** Item size was set by `Instant` (16 bytes, align 8). With a `u32` tick plus a `u8` counter, Item fits in 24 bytes (0004). A per-segment tick removes the contention from a global atomic and makes the smaller layout faster too (0005).
- **The guard's evictee can be dropped.** Evicting the full-probing evictee outright cost no hit-rate in any cell (0007 v002). Cascade re-homing was not keeping useful keys.

## Recurring failures

- **Re-promotion shortcuts.** MRU landing (0001), reuse-aware demotion landing (0002), and the re-admission credit (0003) were redundant with the gate or added churn. The credit lowered hit-rate in every cell.
- **Placing items by free space alone** (balanced/free-slot placement, 0001–0002) never moved hit-rate. It also let cold evictees fill slack.
- **Assuming promotion levers stack.** Size gate, recency window, swap and cascade all fix the same cost: promoting into a slot and evicting a resident that is still useful. Once one lever is present, the others add almost nothing (0005–0007). Whether a lever is "redundant" depends on the lineage. The size gate is redundant under the swap, but under the cascade it protects hm_0 in small caches.
- **Swaps as a tail fix.** A swap never frees a slot, so buckets stay saturated and more ops take the slow full-bucket paths. It helped the tail once (0005) but hurt both tails on the merged parent (0006). The stash→target swap left hit-rate unchanged and slowed latency (0006 v003).
- **Fixed thresholds across cache sizes.** In small caches, keys leave the stash before they can collect an extra hit, so stash-side guards hurt small caches unless they are size-gated (0003–0004). Target-side guards do not have this problem.
- **Latency-only Winners.** Cycles 0003–0007 each show at least one uniform latency shift, often on ops the diff never touches. Most latency-only verdicts are probably noise.

## Open questions

- **Noise baseline (top priority, requested in cycles 0002–0007 and never run).** Re-run an identical baseline, and 0007 v002 next to its parent, on the same host, to size per-op-time variance.
- Does replacing the swap with a single outright eviction on the window-throttled lineage keep the swap's hit-rate, free slots, and cost less than both the swap and the cascade?
- How often do the guarded full-probing and full-target arms actually fire? Which paths dominate get and put p99 now: stash→target, put eviction, or something else?
- Would a cache-sized `PROBE_WINDOW` (wider in small caches) win back the hm_0-small loss?
- Can the non-hottest-hit staleness signal (0005 v003's large-cache gain) be captured without touching the common hit path?
- Does a cheap write-free victim guard recover the large-cache hits lost when the victim comparison was dropped (0002)?
- What limits web_0 at the large cache size, given that placement changes do not affect it?

## Per-policy notes

- **Dash segment (stash / target / probing tiers):** every cycle's changes were here. The current lineage is: threshold-only stash gate → occupancy guard on full targets → full-probing guard (two hottest hits) → swap or cascade on that arm, throttled by a tick-age window (0007 v003 baseline). Small-cache size gate: drop it on the swap lineage, keep it on the cascade lineage.
- **ClassicLRU (evaluated policy):** never reads `timestamp`, so tick or layout changes cannot affect hit-rate. The only cost is maintaining the tick. A per-segment tick, or no tick at all, is preferred.
- **Traces:** reuse-aware gains are largest on hm_0 and at the large cache size. web_0 large is the remaining hit-rate frontier. hm_0 small is the cell most sensitive to removing gates or windows.
- **Caveat:** no variant insight could read overlay source, so the mechanisms above are inferred from the hypotheses, not confirmed from the code.
