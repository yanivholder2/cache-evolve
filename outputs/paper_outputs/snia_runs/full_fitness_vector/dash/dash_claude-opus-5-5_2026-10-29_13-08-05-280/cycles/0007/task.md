# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007

## Winners

- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The three variants were built separately, each on a different parent. v001 made the Item layout change, replacing `timestamp: Instant` with a `u32` tick on top of cycle 0003's `u8` counter. v002 turned on the stash→target full-target guard only in large caches. v003 moved the door-guard idea up one tier, so promotion into a *full* probing bucket now needs two hottest-position hits. All three are Winners. v001 shrinks Item to 24 bytes and is the first metadata-size gain in four cycles. v003 improves hit-rate in every cell. v002 recovers a small amount of small-cache hit-rate and costs nothing elsewhere.

## Patterns

- The door guard is now confirmed at two tiers. Asking for one extra piece of reuse evidence before entering a *full* bucket raises hit-rate, because it cuts the cascades that push reused residents out. This extends a theme seen in every cycle so far: deciding *who enters* matters more than any other lever.
- Whether a guard helps in small caches depends on how long the candidate stays where it is collecting hits. A key in stash is evicted too quickly in a small cache to collect an extra hit. A key in target stays long enough. This is why v003's target-side guard helped small caches without any size gate, while the stash-side guard needed v002's size gate to stop hurting them.
- A guard's hit-rate gain does not automatically make the tail faster. v003 deferred the most expensive move (the full-probing cascade) but did not remove it, and the extra check on the hottest-hit path raised get p99. In cycle 0003, v001's guard improved latency because it removed moves outright rather than delaying them.
- Metadata-size was only ever blocked by `timestamp`. Once that field shrank, the earlier counter narrowing finally showed up in the size. The evaluated policy (ClassicLRU) never reads the timestamp, so hit-rate cannot change. The only remaining question is what it costs to maintain the field. A global atomic tick adds a contended write on every insert.
- Latency noise is still unmeasured. v001's get regression roughly reverses an earlier get gain that was already marked as noise. Most of the small latency moves in this cohort have no mechanism in the code that explains them.

## Open questions

- Do v002's size gate and v003's probing guard add up when combined? Or does the target-side guard make the stash-side size gate unnecessary, since v003 helped small caches with the stash guard left on?
- Can v003's get-tail cost be cut by applying the check more cheaply, or by also shortening the cascade it defers?
- Should the tick be per-segment or per-bucket, or removed entirely for policies that never read it? Any of these would avoid the contended global atomic while keeping 24 bytes.
- Would a stricter version of v003's guard, which resets the count on any non-hottest hit, filter better or worse?
- How large is the per-op-time noise? A re-run of an identical baseline is still missing.

## Continuation

Merge the three Winners into one baseline: v001's 24-byte Item, v002's size-gated stash guard, and v003's full-probing guard. Then check whether the two guards add up or overlap. Replace v001's global `AtomicU32` with a tick that has no contention, or no tick at all for ClassicLRU. Pursue ways to make the full-probing cascade itself cheaper so that v003's get-tail cost goes away. Re-run the baseline before trusting any latency-only change. Drop further work on the stash threshold value alone, because v002 shows little headroom is left there.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants branched separately from cycle 0004 v003, and all three are Winners. v001 merged in the 24-byte Item with a per-segment tick and added the small-cache size gate. v002 replaced the full-probing cascade with a direct two-bucket swap. v003 made the probing guard strict, so any non-hottest hit resets the count. Two variants gave clean gains at no hit-rate cost: v001 improved metadata-size and latency, and v002 cut the get tail. v003 bought a small hit-rate gain with clear get-latency costs.

## Patterns

- Removing work beats deferring it, and this held again. v002's swap removed the cascade outright and gave back v003's get-tail cost. Hit-rate was identical to the last digit in every cell. *Where* the cascade sent displaced entries never mattered. Only *when* promotion fires does.
- Contention, not layout, caused cycle 0004's 24-byte regression. With a per-segment tick, the smaller Item made puts and the get tail faster, probably because the slow paths touch fewer cache lines.
- Any check on the common hit path shows up in median get. This is the third cycle with this lesson. v003's reset sits on the most common hit path, so median get rose even though the reset almost never fires.
- The stash-side size gate is now mostly redundant. Because the target-side probing guard already fixed the small-cache slip, the gate adds only a negligible small-cache gain.
- The guarded full-probing arm seems to fire rarely on these traces. v002's identical hit-rate suggests this, so the remaining tails likely come from other paths: stash→target moves or put eviction. This sits uneasily with v003's hit-rate gain on that same arm, and the two results need reconciling.
- Noise is still unmeasured. v002's and v003's put tails moved with `put` untouched. None of the three variant insights could inspect the overlay source, so their mechanisms are inferred from the hypotheses.

## Open questions

- Do v001 and v002 combine cleanly? They touch separate paths, so the gains should add.
- Can v003's strict reset be moved off the common hit path while keeping its web_0-large gain? One option is to check staleness lazily when the hottest hit happens, for example by comparing ticks.
- How often does the full-probing arm actually fire? Which paths now dominate get and put p99?
- How large is the per-op-time noise? A re-run of an identical baseline is still missing after four cycles of asking.

## Continuation

Merge v001 (24-byte Item, per-segment tick) with v002 (direct swap) as the new baseline. Consider dropping the size gate as redundant complexity. Keep v003's strict-guard idea only if the reset can leave the common hit path, for example as a lazy tick-based staleness check on the rare hottest arm. Otherwise drop it. Next, target the tail paths that remain, stash→target and put eviction. Re-run the baseline before acting on any latency-only change.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005 v001. v001 merged in the direct full-probing swap and dropped the stash-side size gate. v002 replaced the strict reset with a lazy tick-age check on the hottest/full arm. v003 applied the swap idea one tier down, to stash→target. All three are Winners, but none is a clean win. v001 and v002 bought hit-rate with worse get tails. v003 left hit-rate identical and slowed every latency percentile.

## Patterns

- The "swap instead of cascade" lesson did not carry over. In cycle 0005 the probing swap left hit-rate unchanged and cut the tail. On the merged parent it raised hit-rate and hurt both the get and put tails. The likely reason is that a swap never frees a slot, so buckets stay saturated and more gets and puts take the slow full-bucket paths. Keeping items cached helps hit-rate, but the full buckets that result cost latency. The cascade's evictions were doing useful "make room" work.
- The size gate is confirmed redundant. Small-cache cells held or improved without it in v001.
- Lazy checks move cost from the median to the tail, but don't remove it. v002 kept get p50 flat by checking only on the rare arm, but get p99 rose. The recency filter accounts for only part of cycle 0005 v003's large-cache gain. The rest needs the non-hottest-hit signal, which you can't observe without touching the common path.
- Stash→target swapping leaves hit-rate exactly unchanged, the same signature cycle 0005 saw at the probing tier. Either the neighbour retry never mattered, or that branch rarely fires. Firing frequency is still unknown at both tiers.
- Noise is now the binding constraint. v003's slowdown was uniform and hit put, which its diff never touches. That points to environment drift or a shared-helper change, not the mechanism. Put moves in v002 also had no cause in the code. No variant could read its overlay source, so every mechanism here is inferred from the hypotheses.

## Open questions

- Is v003's uniform slowdown real or environmental? A same-session re-run against the parent would answer this.
- Can a swap be combined with occasional eviction (for example, swap only when stash or the bucket has slack) so that hit-rate is kept without saturating buckets?
- How often do the full-probing and full-target guarded arms actually fire?
- Does widening or narrowing `PROBE_WINDOW` change how much of the gain v002 recovers?

## Continuation

Stop adding latency-motivated variants until an identical-baseline re-run measures per-op-time noise. That request has now been ignored for five cycles, and it makes this cohort's latency verdicts unreadable. Keep the size-gate removal. Treat swaps as a hit-rate lever that costs occupancy, not as a tail fix. If you pursue them, test a hybrid that still frees slots sometimes. Drop further recency-window tuning unless it is cheap to test alongside the noise baseline.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
