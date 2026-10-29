## Recurring wins

- **Door guards on full buckets pay off.** Asking for one extra piece of reuse evidence before a key enters a *full* bucket has raised hit-rate at every tier tested: the stash threshold (0001–0002), the full-target occupancy guard (0003) and the full-probing guard (0004). Deciding *who enters* is the strongest lever found so far.
- **Fewer multi-bucket moves helps both hit-rate and latency.** Promotions, demotion cascades and stash churn are one shared cost. Changes that remove moves improve both dimensions. Changes that add moves hurt both (0001, 0003, 0005).
- **Removing work beats deferring it.** Guards that delay a cascade cost tail latency, while dropping the cascade outright gave that cost back (0004→0005). One limit: this only held when hit-rate stayed unchanged (see failures).
- **Keep the common hit path lean.** Taking counter writes and resets off the hottest/target hit path improved median get in three cycles (0002, 0005, 0006).
- **Shrinking Item helps once the real blocker is removed.** Metadata-size was blocked only by `timestamp: Instant`. A `u32` tick plus a `u8` counter gives a 24-byte Item. Once the tick was made contention-free (per-segment rather than a global atomic), the smaller layout also improved puts and the get tail.
- **Most remaining hit-rate headroom is in large caches**, especially hm_0 and web_0 large.

## Recurring failures

- **Placing items by capacity alone does not help.** Free-slot balanced placement had no hit-rate effect, with or without reuse flags (0001–0002).
- **Extra "protect reused items" layers on top of the gate add little.** MRU demotion landing, reuse flags and re-admission credits mostly duplicate what the gate already does. Credits for quick re-promotion actively increased churn and lowered hit-rate everywhere (0003).
- **Fixed stash-side thresholds hurt small caches.** Keys leave the stash too fast in small caches to collect extra hits. The size gate that patched this became redundant once the target-side guard existed, and was dropped (0005–0006).
- **Swaps are not a reliable tail fix.** They never free a slot, so buckets stay saturated. Once hit-rate rose on a merged parent, more operations took the slow full-bucket paths and tails got worse (0006). Stash→target swaps left hit-rate exactly unchanged.
- **Every check on the hot path costs something.** Strict resets cost median get. Lazy tick checks moved that cost into get p99 instead of removing it.
- **Latency-only Winners are unreadable.** Puts and other paths a diff never touched have repeatedly moved by large amounts. The per-op-time noise baseline has been requested in five consecutive cycles and still has not been run.

## Open questions

- How large is the per-op-time noise? An identical-baseline re-run in the same session is the top priority, because it decides how to read every latency verdict so far.
- How often do the guarded full-probing and full-target arms actually fire? Which paths now dominate get and put p99?
- Can a hybrid swap, which still evicts when the bucket or stash has slack, keep the hit-rate gain without saturating buckets?
- Should guard thresholds scale with cache or stash size?
- The non-hottest-hit signal behind cycle 0005's strict-guard gain: can it be captured without touching the common hit path?
- Does choosing a better victim inside a full target add anything beyond the door guards?

## Per-policy notes

- **ClassicLRU (the evaluated policy):** it never reads `timestamp`, so tick or layout changes cannot move hit-rate. Only their maintenance cost matters, and a contended global atomic was the regression. In this design, insert position and promotion rules are coupled: where an item lands sets how fast it climbs tiers.
- **Stash reuse gate (`lfu_counter`-based):** almost all of its value comes from the two-hit threshold. Per-hit counter writes cost median get. The victim comparison gave a small large-cache gain at a small latency cost. `u8` width is enough.
- **MRU landing:** it helps only for demotions and only where the gate is not already saturated. It is not worth pursuing further.
