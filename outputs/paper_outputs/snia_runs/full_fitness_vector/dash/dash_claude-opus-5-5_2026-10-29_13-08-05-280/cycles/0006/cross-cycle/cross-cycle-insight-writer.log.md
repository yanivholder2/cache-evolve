# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-29T14:00:45+00:00

## timeline

### 14:00:49 thinking



### 14:01:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Door guards on full buckets pay off.** Asking for one extra piece of reuse evidence before a key enters a *full* bucket has raised hit-rate at every tier tested: the stash threshold (0001–0002), the full-target occupancy guard (0003) and the full-probing guard (0004). Deciding *who enters* is the strongest lever found so far.\n- **Fewer multi-bucket moves helps both hit-rate and latency.** Promotions, demotion cascades and stash churn are one shared cost. Changes that remove moves improve both dimensions. Changes that add moves hurt both (0001, 0003, 0005).\n- **Removing work beats deferring it.** Guards that delay a cascade cost tail latency, while dropping the cascade outright gave that cost back (0004→0005). One limit: this only held when hit-rate stayed unchanged (see failures).\n- **Keep the common hit path lean.** Taking counter writes and resets off the hottest/target hit path improved median get in three cycles (0002, 0005, 0006).\n- **Shrinking Item helps once the real blocker is removed.** Metadata-size was blocked only by `timestamp: Instant`. A `u32` tick plus a `u8` counter gives a 24-byte Item. Once the tick was made contention-free (per-segment rather than a global atomic), the smaller layout also improved puts and the get tail.\n- **Most remaining hit-rate headroom is in large caches**, especially hm_0 and web_0 large.\n\n## Recurring failures\n\n- **Placing items by capacity alone does not help.** Free-slot balanced placement had no hit-rate effect, with or without reuse flags (0001–0002).\n- **Extra \"protect reused items\" layers on top of the gate add little.** MRU demotion landing, reuse flags and re-admission credits mostly duplicate what the gate already does. Credits for quick re-promotion actively increased churn and lowered hit-rate everywhere (0003).\n- **Fixed stash-side thresholds hurt small caches.** Keys leave the stash too fast in small caches to collect extra hits. The size gate that patched this became redundant once the target-side guard existed, and was dropped (0005–0006).\n- **Swaps are not a reliable tail fix.** They never free a slot, so buckets stay saturated. Once hit-rate rose on a merged parent, more operations took the slow full-bucket paths and tails got worse (0006). Stash→target swaps left hit-rate exactly unchanged.\n- **Every check on the hot path costs something.** Strict resets cost median get. Lazy tick checks moved that cost into get p99 instead of removing it.\n- **Latency-only Winners are unreadable.** Puts and other paths a diff never touched have repeatedly moved by large amounts. The per-op-time noise baseline has been requested in five consecutive cycles and still has not been run.\n\n## Open questions\n\n- How large is the per-op-time noise? An identical-baseline re-run in the same session is the top priority, because it decides how to read every latency verdict so far.\n- How often do the guarded full-probing and full-target arms actually fire? Which paths now dominate get and put p99?\n- Can a hybrid swap, which still evicts when the bucket or stash has slack, keep the hit-rate gain without saturating buckets?\n- Should guard thresholds scale with cache or stash size?\n- The non-hottest-hit signal behind cycle 0005's strict-guard gain: can it be captured without touching the common hit path?\n- Does choosing a better victim inside a full target add anything beyond the door guards?\n\n## Per-policy notes\n\n- **ClassicLRU (the evaluated policy):** it never reads `timestamp`, so tick or layout changes cannot move hit-rate. Only their maintenance cost matters, and a contended global atomic was the regression. In this design, insert position and promotion rules are coupled: where an item lands sets how fast it climbs tiers.\n- **Stash reuse gate (`lfu_counter`-based):** almost all of its value comes from the two-hit threshold. Per-hit counter writes cost median get. The victim comparison gave a small large-cache gain at a small latency cost. `u8` width is enough.\n- **MRU landing:** it helps only for demotions and only where the gate is not already saturated. It is not worth pursuing further.\n"
}
```

### 14:01:09 chat

I've written `cross-cycle-insights.md` from the six-cycle ledger, with the four required sections in order.

**Recurring wins:**
- Guards on full buckets keep raising hit-rate. A key has to show one more sign of reuse before it can enter a full bucket, and this has worked at the stash, target and probing tiers.
- Fewer multi-bucket moves improve hit-rate and latency together.
- Removing work does better than deferring it.
- Keeping checks off the common hit path keeps median get fast.
- The 24-byte Item paid off once the tick stopped being a contended global atomic and became per-segment.

**Recurring failures:**
- Placing items by free space alone doesn't raise hit-rate.
- Extra "protect reused items" layers mostly repeat what the stash gate already does.
- Fixed stash thresholds hurt small caches.
- Swaps keep buckets full, so they don't reliably cut tail latency.
- Latency-only Winners can't be trusted yet.

**Open questions:** The top one is measuring per-op-time noise by re-running an identical baseline. That request has gone unanswered for five cycles, and every latency verdict so far depends on it. The others are how often the guarded paths actually fire, whether a swap that still evicts when there's room could work, and whether guard thresholds should scale with cache size.

**Per-policy notes:** These cover ClassicLRU, the stash gate driven by `lfu_counter`, and MRU landing. ClassicLRU never reads `timestamp`, so tick and layout changes can't move its hit-rate; only their upkeep cost matters.


---

- finished: 2026-10-29T14:01:10+00:00
- duration: 24.47s
- status: ok
- input_tokens: 4
- output_tokens: 2195
- cache_read_tokens: 40393
- cache_write_tokens: 12118
- cost_usd: 0.148939
