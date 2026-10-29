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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cohort tried to untangle cycle 0006's bundled changes. v001 dropped the size gate on top of 0006 v002's cascade. v002 replaced the guarded full-probing cascade with a single outright eviction. v003 put the `PROBE_WINDOW` age check on top of 0006 v001's swap. The Winners are v003 and v001, and both are mostly latency wins. v003 recovered much of the swap's get and put tail cost and gave up a little hm_0 hit-rate. v001 slightly improved get p99 and lost a little small-cache hit-rate. v002 is not a Winner, but on hit-rate it may be the most useful result.

## Patterns

- The two hypotheses that said "the gains will stack" were both refuted, and they fail in the same way. Gate removal only helps when a swap is present (v001). The recency window only helps when a cascade is present (v003). Each lever fixes the same underlying cost, which is a promotion that evicts a still-useful resident, so once one lever is in place the other adds nothing. Promotion levers on the same arm overlap; they do not stack.
- The size gate is redundant only on the swap lineage. Under the cascade it was holding up hm_0 in small caches. Cycle 0006's claim that the gate is redundant should be read with that condition attached.
- The probing evictee is disposable. With v002, dropping it outright cost no hit-rate in any cell. The cascade's re-homing was not keeping useful keys, so the cheap evict-and-free-a-slot path is safe on hit-rate. This also resolves cycle 0006's dilemma: you can free a slot without the cascade's multi-bucket work.
- A recency window is better used as a throttle on how often the swap fires than as a hit-rate filter. The swap stays on the get path but fires less often. This is the first lazy check that brought the tail down, not just moved cost into it.
- Noise is still the binding constraint. v002's regression was uniform across ops and percentiles, including paths its change never touches, so it has to be an artifact. That makes it the third cycle in a row with an unexplained uniform shift. Again, none of the variants could read its overlay source.

## Open questions

- Is v002's latency regression real? Re-run it next to its parent on the same host.
- Would v002's single eviction, put on the swap/window lineage in place of the swap, keep v003's hit-rate and also free slots? That would combine the hit-rate of the swap, the occupancy of the cascade, and a cost lower than either.
- Could a window sized to the cache (wider in small caches) win back v003's hm_0-small loss?

## Continuation

Take v003 as the baseline, and drop further "stack two promotion levers" hypotheses, because they overlap. Top candidate: replace v003's swap with v002's single outright eviction while keeping the window as the throttle. Re-run v002 against its parent on the same host before trusting any latency verdict. This is the sixth cycle asking for an identical-baseline noise run, and it should be done in the next cohort, not deferred again.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v003, the window-throttled swap lineage. v001 replaced the swap with a single outright eviction of the probing resident. v002 doubled `PROBE_WINDOW`. v003 changed stash victim choice to prefer never-hit entries. All three are listed as Winners, but only v001 brought a useful result: a large-cache hm_0 hit-rate gain, with its latency verdict lost to another uniform shift. v002's hit-rate gain was negligible, and it was bought with a real get-latency cost. v003 lost hit-rate in every cell.

## Patterns

- The eviction-instead-of-swap result now holds on two lineages. Dropping the probing evictee cost nothing under the cascade in cycle 0007, and under the window throttle it cost nothing and slightly helped large-cache hm_0. Keeping the demoted resident is worthless. Freeing a slot is at least neutral and possibly helpful.
- `PROBE_WINDOW` is a latency throttle, not a hit-rate lever. Widening it let many more hottest hits into the costly swap, and the median rose as well as the tail, so the swap is not a rare event once admitted. The parent's window already sits near the useful end of that trade-off. The hm_0-small dip has some other cause.
- The stash's LRU-position eviction is already the right scan-resistant policy. Protecting one-hit stash entries filled the stash with keys that are rarely reused and starved fresh arrivals before the gate could judge them. Small caches were hurt most, and web_0-large still did not move. This was the first test of "who leaves the stash", and it shows the lever cuts the wrong way unless the protection ages out.
- Any added scan or check shows up exactly where it runs. v003's put cost appeared on the stash-full put path, and v002's cost appeared on the get path. This is consistent with the recurring "work on a path shows on that path" lesson.
- Noise is still unmeasured. v001's near-uniform slowdown, including the median of an untouched put, has the same signature as cycle 0007 v002. This is the fourth cycle running with an unexplained uniform shift, and still no identical-baseline re-run has been done. As before, no variant could read its overlay source. v002 also flags a possible implementation side effect, a per-get bound computation, that could inflate its median.

## Open questions

- Does v001 actually improve latency once it is timed against v003 on the same host in the same session?
- What is holding back hm_0-small on the swap lineage, if the window is not the cause?
- Would an aging bound on stash protection, where one old hit does not shield an entry for long, recover any of the web_0-large gain, or is the stash-exit lever dead?
- Does narrowing the window on v001's eviction lineage cut latency further without costing hit-rate?

## Continuation

Adopt v001 (window throttle plus single eviction) as the baseline on hit-rate grounds. Before trusting any latency verdict, run v001, v003 from cycle 0007, and an identical copy of the parent together on one host. This request has gone unmet for seven cycles, and it has to be a hypothesis of its own in the next cohort. Drop window widening. Drop unbounded stash-victim protection. Look for hm_0-small and web_0-large hit-rate in the arm's mechanics or in admission, not in throttle tuning.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cohort finally ran the noise control. v001 is an identical copy of cycle 0007 v003, and v002 and v003 are two changes on the 0008 v001 eviction lineage: v002 halved `PROBE_WINDOW`, and v003 added aged, bounded stash-victim protection. All three are Winners. v001 is valuable as evidence, not as code. v003 is the real result: the first stash-exit change to gain hit-rate (hm_0 at both sizes) with no loss in any cell. v002 is hit-rate neutral to slightly positive.

## Patterns

- Cross-session latency is not trustworthy. The unchanged copy slowed sharply and uniformly, put included. v002 and v003 sped up uniformly against their parents, including on paths they never touch. So the "uniform shift" seen in cycles 0005–0008 was environmental drift. Cycle 0008 v001's apparent get regression is probably drift too. From now on, latency can only be scored against a same-session control.
- Under drift, the variant writers fell back on comparing ops within one run, and that turned out to be a usable signal. In v003 the stash-full put path shows no tail inflation relative to get. That suggests bounding protection to two slots avoided cycle 0008 v003's put cost.
- The stash-exit lever is alive once protection ages out. Unbounded protection lost everywhere. Recency-gated protection with a single reprieve and no MRU refresh gained on hm_0 and lost nowhere. The gain depends on the trace, not on cache size: it shows up where stashed keys get short-range reuse (hm_0) and not on web_0. That is the opposite of what H03 predicted. web_0-large is still untouched by any lever tried so far.
- `PROBE_WINDOW` is now confirmed as hit-rate-inert in both directions on the eviction arm. Narrowing it cost nothing and may help marginally, which fits the earlier finding that the demoted resident is disposable. It is a pure latency knob, and its latency effect is still unmeasured.
- Again, no variant could read its overlay source. Mechanisms are inferred from the hypotheses.

## Open questions

- Did v002 actually lower get latency? This needs a direct same-session read of v002 against v001. None of the variant insights made that comparison. The control was also the swap lineage, not the eviction parent.
- In this session, is the eviction lineage (0008 v001) faster than the swap lineage (0007 v003)? The cross-session evidence hints that it is, but this is unconfirmed.
- Does v003's protection stack with v002's narrower window? They act on different paths (stash exit vs. probing arm), so unlike earlier promotion levers they may not overlap.
- What would move web_0-large? Stash exit, the window, and arm eviction have all left it flat.

## Continuation

Adopt v003 as the hit-rate baseline. From now on, every cohort must include an identical-copy control of its own parent, and latency verdicts may only be scored against that control. Next, try combining v003 with v002's narrower window, since they act on different paths. You could also tune `STASH_PROTECT_AGE`, judging it on hm_0. Drop further `PROBE_WINDOW` hit-rate hypotheses. For web_0-large, look at admission (the stash gate or the door guard), not at stash exit or the probing arm.


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
