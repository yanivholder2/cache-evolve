# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed promotion and placement inside the Dash segment and left Item layout alone. They tried MRU landing (v001), free-slot balanced placement (v002) and a stash reuse gate driven by `lfu_counter` (v003). The Winners were the gate (v003), which improved both hit-rate and get tail, and balanced placement (v002), which won on latency only.

## Patterns

- Hit-rate and per-op time move together through one shared cost: how many multi-bucket moves (promotions plus demotion cascades) each hit triggers. Changes that reduce those moves (v002, v003) were faster. The change that added moves (v001) was slower even though each insert was cheaper.
- Hit-rate improved only when a change decided what to keep based on reuse evidence (v001 protects items that were hit; v003 admits only items with proven reuse). Changes that decided placement by free space alone (v002) did nothing under pressure and kept cold items when there was slack.
- In this design, insert position and promotion rules are coupled. Where an item lands sets how quickly it climbs tiers on its next hit, not only its eviction order. The hottest-in-target → probing rule makes this coupling worse.
- The gains from reuse-aware changes were largest on hm_0 and at the large cache size, where there is room for a reused working set once churn stops.
- Fixed bookkeeping on every hit (counter writes, comparisons with the victim) costs a little on median get even when it cuts the tail a lot.

## Open questions

- Is v003's median get penalty caused by the per-hit counter increments in target/probing, or by the victim comparison at admission?
- Would v001's protective `demote_pos = Back` still raise hit-rate if `promote_pos` stays Front, or if it is combined with v003's gate, without the extra promotion churn?
- Is v002's latency gain real or partly noise? Could the cheap no-evict move into a free slot be kept for demotion only, without letting cold evictees fill probing?
- Should the gate threshold grow with cache size?

## Continuation

Build on v003's reuse gate as the main line. Then try adding v001's MRU landing for demotions only, and/or requiring reuse before target → probing, to gain hit-rate without extra churn. Re-measure v002's chain-shortening and keep it only if the latency gain holds and free slots go to hot items, not cold evictees. Drop pure capacity-based balanced placement as a hit-rate strategy.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants built on cycle 0001's stash reuse gate. v001 removed the gate's extra parts (the per-hit counter writes and the victim comparison), and v002 added reuse-aware MRU landing for demotions. The third, v003, added a reuse flag to v002's free-slot placement. The Winners are v001, which got a faster get path for a small loss of large-cache hit-rate, and v003. v003's own insight reports no improvement on any dimension, so its place among the Winners should be read with caution (see Open questions).

## Patterns

- The cycle-0001 gate splits into separate parts. The two-hit stash threshold gives almost all of the hit-rate gain. The counter writes on normal-tier hits cause most of the median get cost. The victim comparison buys a small large-cache hit-rate gain, mainly on hm_0, for a small get-latency cost. Each part can now be priced on its own.
- Once the gate is in place, other "protect reused items" mechanisms add little. v002's MRU demotion landing helped only slightly, and only where the gate had not already saturated the benefit (web_0 large). v003's flag-limited free-slot placement left hit-rate unchanged in every cell. Extra reuse evidence mostly duplicates what the gate already captures.
- Any reuse check on the common target-hit path costs a little median get, whether it is a counter read, a compare or a conditional store. The summary latency statistic is sensitive to this, so the common hit path should stay free of bookkeeping.
- Large cache sizes are where the remaining hit-rate is won or lost. Both the victim comparison's benefit and the cost of dropping it showed up there, not in the small-cache cells.
- The per-op-time benchmark looks too noisy to separate small changes. v003 regressed by more than its code change can plausibly explain, and that casts doubt on v002's cycle-0001 latency gain.

## Open questions

- Is the per-op-time measurement stable? Re-running an identical variant would show how large the noise is before more latency conclusions are drawn.
- Can a cheaper victim guard win back v001's lost large-cache hits without per-hit counter writes? For example, a single-bit check, or a guard active only at large sizes.
- Why is put p99 higher in v001? Is it from the extra stash churn of unconditional promotion into a full target, and can it be reduced?
- If free-slot placement does not drive hit-rate, what limits web_0 at the large size?

## Continuation

Use v001 (threshold-only gate, lean hit path) as the new baseline. Next, try a cheap, write-free victim guard, possibly only at large cache sizes, to recover hit-rate there. Drop the free-slot placement line (from cycle-0001's v002) and MRU demotion landing, since both are redundant with the gate. Before trusting small latency differences, check measurement noise, for example with a re-run of the baseline.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle 0002's lean threshold-only gate. v001 added an occupancy guard (a third stash hit is needed to enter a *full* target). v002 made the first change to Item layout by narrowing `lfu_counter` to `u8`. v003 gave demoted target residents a re-admission credit. All three are Winners, but only v001 has a mechanistic gain: better large-cache hit-rate and cheaper puts and get tail. v002 left metadata-size unchanged, and v003 lowered hit-rate in every cell. Their latency gains are most likely measurement noise.

## Patterns

- Deciding *who enters* a full target works. Making it *cheaper to come back* does not. v001's guard blocks churn at the door and recovers large-cache hits. v003's credit speeds up swaps between reused residents that share a full target, which adds stash inserts and evicts fresh items that are still waiting for their second hit.
- Stash churn is the shared cost behind both hit-rate and put latency. v001 cut forced demotions into stash and had the clearest put improvement. v003 raised churn and lost hit-rate everywhere. This extends cycle 0001's finding that the number of multi-bucket moves drives both dimensions.
- Protection added by the gate trades large-cache gains against small-cache losses. In small caches, stash residence is too short for a key to collect an extra hit, so a fixed threshold helps one size and hurts the other.
- Metadata-size is set by `timestamp: Instant`, which takes 16 bytes with align 8. The counter is not the constraint. Any field of 8 bytes or less disappears into alignment padding, so narrowing the counter alone cannot shrink Item.
- The per-op-time noise problem is now confirmed in practice. Two variants whose diffs cannot explain their latency moves (put was untouched in both) showed large drops. Latency-only "wins" can't yet be told apart from variance.

## Open questions

- How much noise does per-op-time carry? No variant has re-run an identical baseline yet, and that blocks trust in any latency-only Winner.
- Should the full-target threshold scale with cache or stash size, or apply only when the stash is large enough for three hits to be plausible?
- Can `timestamp` become a ≤4-byte tick, or be dropped for policies that never read it, so that Item fits in 24 bytes without changing hit-rate?
- Does better victim choice inside a full target add anything on top of v001's door guard?

## Continuation

Make v001 (the occupancy guard on full targets) the new baseline. Next, try a size-aware version of its threshold to win back the small-cache slip. Keep v002's `u8` counter as a prerequisite, and attack metadata-size by narrowing or removing `timestamp`. Drop re-admission credits and any other "faster re-promotion" idea. Before accepting another latency-only Winner, re-run the baseline so the size of the measurement noise is known.


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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
