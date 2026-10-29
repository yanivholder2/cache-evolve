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
