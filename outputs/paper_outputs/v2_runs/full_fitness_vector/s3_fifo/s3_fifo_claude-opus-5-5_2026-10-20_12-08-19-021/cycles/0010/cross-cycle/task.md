# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three baseline-parented hypotheses attacked hit-rate on workload-a through different levers: a looser small→main graduation rule (H01), ghosting main-queue victims (H02), and ARC-style adaptive small/main sizing (H03). Only H03 (v003) was evaluated and it won. v001 (H01) and v002 (H02) were rejected because the simulator build failed with HTTP 429s from Maven, not because of anything in their code, so their levers are still untested.

## Patterns

- Changing queue *capacity* (how much room small and main get) and leaving the per-key movement rules alone improved hit-rate. So the fixed 25/75 split is not optimal for workload-a.
- No dimension got worse. Per-entry metadata stayed flat, which means size adaptation costs nothing per entry.
- Latency got better too, but nothing in the change explains it. It may just be fewer miss scans because the hit-rate went up, or it may be noise.
- The two resize triggers are lopsided ("main full on graduation" fires almost all the time in steady state). The split probably settles at a balance point set by the clamps and the ghost-hit rate, rather than truly adapting.

## Open questions

- Is the v003 gain coming from the adaptation itself, or just from landing on a better fixed split? The next variant should log or report where `small_size` settles.
- Was the latency improvement real? It needs re-measuring.
- Do H01 (graduation threshold) and H02 (ghosting main victims) help on their own? Both are untested only because of infrastructure failures.

## Continuation

Re-submit H01 and H02 essentially unchanged, preferably with v003 as the parent, since both are movement-rule changes that should stack with v003's sizing lever. Probe the H03 mechanism as well: try a fixed split near where `small_size` settles, or make the grow-small trigger fire less often. Don't credit adaptive sizing for latency gains until they are reproduced.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three hypotheses used cycle 0001's v003 (the adaptive split) as their parent. Two re-tried movement rules from cycle 0001 that never got evaluated: H01 loosened the graduation rule and H02 ghosted main-queue victims. The third, H03, was a pure data-structure refactor that swapped linear scans for hashed indexing. v003 (H03) is the clear winner: hit-rate stayed identical, and both `get` and `put` latency dropped sharply. v002 (H02) is the other Winner, but its own insight reports it lost to its parent on both hit-rate and latency, so it probably made the cut on relative rank within a weak cohort, not on merit.

## Patterns

- Neither movement-rule change improved hit-rate on top of the adaptive split. H01 was flat and H02 was slightly worse. Both look like they interact with the split adaptation: more graduations or more ghost hits push the split around, and that seems to cancel or reverse the intended effect. Neither variant logged where `small_size` ends up, so this is still unconfirmed.
- The latency numbers from earlier cycles were dominated by O(capacity) scans. v003 confirms that cycle 0001's "unexplained" latency gain came from the scans and not from the policy. The `get` regressions seen in v001 and v002 are most likely that parent artifact reverting, not effects of their rules.
- The `metadata-size` metric only counts `Entry<K,V>`, so it misses the cost of structural changes. v003 increased real per-resident and per-ghost overhead and the dimension reported no change.
- Every change that adds ghost traffic is expensive under linear-scan ghosts. That cost no longer applies on the hashed baseline.

## Open questions

- Where does `small_size` actually settle, and are the movement-rule changes being absorbed by the adaptation? No variant has reported this yet.
- On the hashed v003 base, do H01 or H02 still cost latency, or was that cost only a side effect of the scans?
- Would a separate main-victim ghost that does not feed the split adaptation recover any hit-rate from H02's idea?
- Is the adaptive split better than the best fixed split? This is still untested.

## Continuation

Use cycle 0002's v003 (hashed) as the parent and as the latency baseline from now on. Do not credit latency claims measured on older scan-based parents. Drop the plain H01 threshold change. Retry H02 only in its decoupled form: a separate ghost for main victims that does not trigger "grow main". Any hypothesis that touches the split should instrument or report the value `small_size` settles at, and should compare against a fixed split near that value.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three hypotheses used cycle 0002's hashed v003 as their parent, and each made a different kind of change:

- **H01** replaced the lopsided adaptation with ARC-style two-ghost signals.
- **H02** replaced the key-cloning ghost with a fixed ring of fingerprints.
- **H03** replaced the whole adaptive policy with the paper's static 10/90 split and its `freq >= 1` graduation rule.

All three are Winners. Only v002 (H02) improved on its parent on both hit-rate and latency. v001 and v003 each lost a little hit-rate to the parent and made it into the Winners on relative rank in the cohort.

## Patterns

- **The incumbent policy has now held up three cycles in a row.** v003's lopsided adaptation with two-touch graduation has outlasted correctly directed ARC signals (v001) and the published static setup (v003). So the adaptive gain is not just an escape from a bad 25/75 start. Something in the "adaptation + `freq > 1` graduation" package is earning a small but real margin.
- **Both losing variants changed two levers at once.** v001 changed the signal direction and also let main-ghost hits bypass `small`. v003 changed the split and the graduation rule together. In neither case can we tell which lever cost the hit-rate.
- **The ghost behaves like a policy knob, not plumbing.** v002 was meant to be policy-neutral. But its ring shortens the ghost's effective memory: holes left by taken entries still use up positions. That made ghost hits rarer, and the split drifted in a favourable direction. In the other direction, v001's extra ghost traffic pushed the split around in a way that hurt. Ghost size and contents feed the adaptation directly.
- **Most `put` latency gains are real; `get` gains may be noise.** `put` gains trace to specific removed work: the clones and `retain` compaction in v002, and the resize branches in v003. But v001 and v003 also report `get` gains with `get` code byte-identical to the parent. That points to benchmark variance or host drift, which also casts doubt on part of v002's `get` gain.
- **`metadata-size` still misses all ghost cost.** It hid v001's second ghost, v003's larger ghost, and v002's real heap savings.

## Open questions

- **Which half of each two-lever change caused its loss?**
  - For H03: fixed 10/90 split with the parent's `freq > 1` graduation (no freq reset).
  - For H01: keep the main ghost but reinsert its hits into `small`, or restore v003's small-ghost direction.
- **Where does `small_size` actually settle?** No variant has reported it yet, even though cycle 0002 asked for it. Does the ring ghost move it?
- **Is ghost size or hole handling a tunable lever for hit-rate on the ring base?**
- **Are the `get` latency gains real?** This needs a same-host re-baseline of the parent.

## Continuation

Adopt v002 (the fingerprint ring) as the new parent. It wins on both axes and keeps the incumbent policy. Drop wholesale policy replacements and ARC-direction rewrites. Instead, test one lever per variant: graduation rule alone, fixed split alone, ghost capacity or hole handling alone, and main-ghost reinsertion into `small`. Every variant must report where `small_size` settles. Credit `get` latency changes only when the `get` path itself changed.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants used cycle 0003's v002 (the fingerprint ring) as their parent, and each was meant to change one lever. H01 halved the ghost ring. H02 froze the split at a static 10/90 with `freq > 1` graduation kept. H03 was a policy-neutral cleanup of the `put` path. All three are Winners, but only v003 (H03) clearly improved: `put` got a little faster and hit-rate was identical. v001 moved hit-rate barely above zero, and v002 lost on both hit-rate and latency, so both made the cut on rank in a small cohort.

## Patterns

- **The adaptive split has now held up four cycles in a row, and it soaks up changes to other levers.** Halving the ghost barely moved hit-rate, because the adaptation and its `capacity/16` / `capacity/2` clamps absorbed most of the change. Freezing the split lost hit-rate even with the incumbent graduation rule kept. Together with cycle 0003's static-split loss, this is the strongest evidence yet that the adaptation itself earns its margin and is not just an escape from a bad starting split.
- **"One lever" variants keep coupling ghost size to the split.** v002 tied `ghost_cap` to the new main size, so the ghost also grew. That repeats the confound from cycle 0003. Any change to the split has to pin ghost size on its own.
- **`put` gains come from cutting machine work, not hashing.** v003's gain traces to swapping the ring's modulo wrap for a compare and to inlining the eviction functions. Its premise of redundant hashing was a miscount: the parent already hashed and cloned as little as the refactor did. Count hashes and clones in the parent before you claim them as a lever.
- **`get` latency is noise.** All three variants left `get` byte-identical, and all three saw it move, mostly upward. Because the headline latency metric tracks `get`, a `put`-only refactor cannot improve the summary.
- **`metadata-size` still misses ghost heap.** It hid both v001's ghost shrink and v002's ghost growth.

## Open questions

- Where does `small_size` actually settle? Four cycles have asked for this, and no variant has reported it.
- Does a static 25/75 split (the parent's starting point), with the ghost pinned at 75%, match the adaptive split? That would isolate the adaptation from the split ratio and the ghost size.
- Would loosening or tightening the adaptation clamps turn ghost length back into a lever that matters?
- Is the tiny gain from a halved ghost real, or within noise?
- Would a single-probe miss path (a precomputed-hash or raw-entry index) give a real `put` gain beyond v003's?

## Continuation

Adopt v003 as the new parent: it keeps the incumbent policy with a faster `put`. Stop trying to shrink the ghost, and stop replacing the adaptation with any static split unless ghost size is pinned at the parent's value. Productive next levers are the adaptation's clamps and trigger strength, main-ghost reinsertion into `small` (still untested), and a true single-probe index for `put`. Every variant must report where `small_size` settles, and must not credit `get` latency changes when the `get` path is untouched.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants used cycle 0004's v003 as their parent. Each changed one lever:

- **H01** lowered the small-queue floor clamp from `capacity/16` to `capacity/64`.
- **H02** readmitted ghost-hit keys into main with `freq = 1`.
- **H03** moved values into the hash map to get a single-probe `get`.

All three are Winners, but each won something different. v003 ranked first on `metadata-size`, the first real drop in that metric so far, but `get` got slower. v001 is the only variant that improved hit-rate, and only slightly. v002 lost on both hit-rate and latency and made the cut on rank.

## Patterns

- **The lower clamp binds, but only weakly.** Hit-rate moved when the floor was loosened, so the split does reach `capacity/16` on workload-a. The gain was small, which means either the split only sits at the floor part of the time or the benefit of a bigger main levels off quickly. This is the first cycle where the adaptation's bounds turned out to be a live lever. Movement-rule tweaks in earlier cycles did not.
- **Extra credit on readmission hurts.** Admitting a key straight to main from the ghost is already its reward. Adding `freq` on top kept one-time-reuse keys in main for an extra lap, pushed out keys whose reuse was seen through real `get`s, and added rotation work to `evict_main`. Together with cycle 0002's ghosted-main-victims loss, this suggests that changes making the ghost path stronger tend to backfire.
- **The `get` cost is the probe's memory footprint, not the dependent load.** Putting the payload inline widened the hash buckets to a size that is not a power of two, and `get` got slower at both p50 and p99. That is the first `get` move in several cycles that noise does not easily explain. Counting dependent loads predicted the wrong direction.
- **A slimmer `Entry` and a fast `get` pull against each other only when the entry is stored inline in the probe table.** Shrinking the entry inside the slab could reduce `metadata-size` without widening the index.
- **`small_size` is still unreported.** v001 and v002 both added the instrumentation, but neither writer could see its output. So we still cannot tell whether v002's loss came from a split shift or from main residency alone.

## Open questions

- What does `h01_split_report` show: where does `small_size` settle, and how often did the old floor block a shrink? This needs to surface in the variant insights, not just the variant notes.
- Is the grow-small trigger, which pulls the split back up, now the stronger lever than the floor?
- Is v001's `get` regression noise? Its `get` path is byte-identical to the parent's.
- Can the slab keep a key-free 16-byte `Entry` while the index stays at the parent's narrow `(K, u32)` buckets?
- Would conditional readmit credit (only on a second ghost hit, or only while small is at its floor) behave differently from the blanket version?

## Continuation

Use v001 (the `capacity/64` floor) as the policy parent, since it is the only hit-rate gain. Don't lower the floor any further. Probe the grow-small trigger strength or the upper clamp next, one lever at a time with ghost size pinned. Drop blanket readmit-`freq` credit and inline-payload index layouts. To shrink `metadata-size`, try a key-free `Entry` that stays in the slab behind the narrow index. Each variant must get its settled `small_size` figure into its insight, and must not credit `get` changes unless the `get` path or the index layout changed.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle combined the two cycle-0005 incumbents and probed one new policy lever. H01 put the `capacity/64` floor onto 0005 v003's inline, slab-free layout. H02 moved the key out of the slab `Entry` into a parallel `keys` array behind the narrow `(K, u32)` index, on top of 0005 v001. H03 halved the grow-small trigger with `GROW_SMALL_EVERY = 2`. All three are Winners. v002 is the only one that improved without regressing anything: `get` got faster, `Entry` reached 16 bytes, and hit-rate was identical. v001 matched the floor's hit-rate at minimum metadata-size, but its latency regressed. v003 was neutral on hit-rate and made the cut on rank.

## Patterns

- **Policy and layout are independent levers.** The floor's hit-rate gain reproduced exactly on a different storage layout (v001), and the layout refactor kept the policy exactly the same (v002). So winning levers can be stacked freely across lineages.
- **Splitting hot and cold data is the layout lever that works.** Inlining the payload in the probe table slowed `get` (cycles 0005 and 0006, v001). Moving cold fields such as the key out of the array `get` reads, while keeping the index narrow, sped `get` up (v002). The `get` path should touch only `{value, freq}`.
- **`metadata-size` now overstates savings.** It measures only `size_of::<Entry>`. In v002, the `Option<K>` key array gives back roughly the bytes the 16-byte `Entry` saved, so the real per-slot footprint barely moved. Per-cache fields such as v003's padded `u8` show up in a separate row. Treat this metric as a proxy for the hot array, not for total memory.
- **The grow-small side of the split is not a lever on workload-a.** Damping the grow-small trigger did nothing to hit-rate. Combined with the weak but real gain from the floor, this suggests the ghost-hit shrink side sets the balance point. The "bigger main helps" theory has now run out of cheap triggers to pull.
- **Latency noise is still large, especially at p99.** v001 and v003 both saw `get` move in opposite directions with byte-identical `get` code. Only v002's `get` change has a code-path explanation, and even part of it may be the parent's tail settling back.
- **`small_size` is still unreported.** v003 refers to its `h03_split_report` but gives no settled value. Six cycles have now asked for it.

## Open questions

- Does the settled split actually move under the /64 floor or the damped grow-small trigger? `settled_small`, `grow_events` and `grows` need to appear in an insight.
- Would a niche-free `keys: Vec<K>`, with liveness tracked through `free`, really shrink the per-slot footprint? And would it keep v002's `get` gain?
- Is v001's `put` regression a real CLOCK-walk cost from a longer `main`, or noise? That needs a same-run re-baseline against its parent.
- Does v002's delayed `Drop` of stale values matter for non-trivial `V`?

## Continuation

Adopt v002 as the parent. It has the floor policy, the narrow index and split hot/cold arrays, and it wins on `get` with no regressions. Drop the inline-payload layout (v001's lineage) and the grow-small damping lever. Next, try a niche-free `keys` array to cut the real per-slot footprint, and a same-run latency re-baseline. Only test further policy levers on the ghost-hit shrink side, one at a time with the ghost pinned. Every variant must put its settled `small_size` figure in its insight.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants used 0006 v002 as their parent. Two were policy-neutral trims and one changed policy:

- **H01** moved `freq` into a parallel `freqs` array, which shrinks `Entry` to 8 bytes.
- **H02** doubled the ghost-hit shrink step, with the ghost pinned.
- **H03** skipped the `freq` store on `get` when the counter is already saturated.

All three are Winners, but none improved without a regression. v001 ranks first on a real per-slot `metadata-size` drop. v003 and v002 made the cut on rank: hit-rate was flat across the whole cohort, and every variant regressed on latency.

## Patterns

- **The benchmark baseline drifted this cycle.** `put` slowed in all three variants, including v003, whose `put` path is byte-identical to the parent's. So this cycle's latency deltas can't be credited to the code, in either direction. That includes v001's `get` loss, even though it has a plausible cause: a second cache line on every hit.
- **Splitting hot fields from each other is a different trade from splitting cold fields out.** Moving the key out (0006) helped `get`. Moving `freq` out splits two fields that every hit reads together. The metadata saving is real, but at this capacity the slab already fits in cache, so there was little locality to win back.
- **Neither trigger ratio is a lever on workload-a.** Cycle 0006 found the grow-small side inert, and the ghost-hit shrink side is now inert too. If the split responds to anything, it is the bounds (the `capacity/64` floor) or the ghost sizing, not how fast it moves between them.
- **Store-elision micro-trims on `get` are below what this benchmark can resolve.** At this measurement noise, a single-threaded L1-resident store costs too little to show up.
- **`small_size` is still unreported.** This is the seventh cycle in a row. All three writers said the split-report output was not in the artifacts they could reach. The variants did generate it, but it never reaches the insight writer.

## Open questions

- Is v001's `get` regression real, or drift? Only a same-run parent-vs-child benchmark can tell.
- Can `freq` be packed into spare bits next to the value or the index entry? That would keep the 8-byte saving without a second access per hit.
- Does `settled_small` sit at the floor under both shrink steps (`floor_blocks`)? If so, the floor is the binding limit.
- Is the pinned ghost capacity, rather than the split bounds, what caps hit-rate now?

## Continuation

Keep 0006 v002 as the latency parent, and carry v001's `freqs` split only as a metadata-size branch until a same-run re-baseline clears its `get` cost. Prefer a bit-packed `freq` over a separate array. Drop trigger-step tuning on both sides, and drop `get` store-elision. The next policy lever should be the ghost capacity (held relative to the initial main size) or a test of whether the floor binds. Before trusting any latency claim, the loop must (1) benchmark parent and child back-to-back in the same run and (2) copy the `h0x_split_report` stdout into the eval artifacts the insight writers can read.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle had one layout change and two single-lever policy probes:

- **H01** packed the 2-bit `freq` into spare high bits of the index value. Its parent was 0007 v001.
- **H02** grew the ghost to the full `capacity`. Its parent was 0006 v002.
- **H03** raised `MAX_FREQ` from 3 to 7. Its parent was 0006 v002.

All three are Winners. v001 is the only one that improved without regressing anything. `get` got faster, it keeps the 8-byte `Entry` with no per-slot cost for `freq`, and hit-rate is unchanged. v003 gained a little hit-rate but paid for it in `put` and `get` latency. v002 lost on both hit-rate and `put` tail latency and made the cut on rank.

## Patterns

- **Bit-packing resolves the conflict between hot-field splitting and latency.** In 0007, moving `freq` into a separate array cost `get` a second memory access. Putting it in the index bucket that `get` already probes gives the metadata saving with no extra access. The rule now holds across three cycles: the `get` path should touch only memory it already reads, which is the probed bucket plus the value.
- **Making the ghost path stronger has backfired again.** A larger ghost readmitted keys whose reuse distance is too long to survive in main, and those keys pushed out better residents. Earlier failures from the ghost path were ghosting main victims (0002) and readmit-with-`freq` (0005). This is the third, so the evidence now suggests ghost capacity is not what limits hit-rate.
- **Retention depth is a real hit-rate lever, but an expensive one.** `MAX_FREQ` is the first lever since the floor to move hit-rate in the right direction. The cost lands entirely on eviction work: longer CLOCK drains slow `put` and, through cache pressure in the mixed stream, also slow `get`. Hit-rate gains on workload-a keep coming in small amounts, and only from the bounds or the depth of the policy, never from trigger rates.
- **Eviction-path costs reach `get`.** v003's `get` code was identical to its parent's, yet `get` regressed. In a mixed benchmark, `put`-side sweep work counts against the summary `get` metric.
- **`small_size` and the split reports are still unreported.** This is the eighth cycle in a row. No writer could read any `h0x_split_report` output, so `floor_blocks` and the rotations-per-eviction figures stayed unverified. v001's `put` gain also still lacks a same-run re-baseline.

## Open questions

- Can the `MAX_FREQ` hit-rate gain be kept with a cheaper drain? Options are halving on each pass instead of decrementing, or a cap of 4–5. A cap of 7 needs 3 bits, which no longer fits v001's 2-bit packing. Taking a 29-bit slot id would make room.
- Does a larger ghost pin small at the floor? Checking `floor_blocks` would tell.
- Is v001's `put` improvement real, or the 0007 drift reversing?

## Continuation

Adopt v001 (packed `freq` in the index) as the parent for both latency and metadata. Drop ghost-capacity growth and any change that strengthens readmission; if the ghost is probed again, try it smaller or make readmission stricter. Pursue the frequency-cap lever only together with a cheaper main drain, such as a halving decay or a cap of 4–5. Packing it as 3 bits into the v001 layout is fine. The loop infrastructure still has to benchmark each parent and child in the same run and get the split-report stdout to the insight writers.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants used 0008 v001 (packed `freq` in the index) as their parent:

- **H01** raised `MAX_FREQ` to 7 as a 3-bit packed counter and drained it by halving.
- **H02** replaced hashbrown with a custom linear-probe `(K, u32)` table.
- **H03** sent ghost hits back through small with freq 1 instead of straight to main.

All three are Winners only on rank. Each one regressed something, and none beat the parent cleanly. v001 cut the `put` tail but lost a little hit-rate. v002 kept hit-rate the same and trimmed the `get` tail, but `get` p50, `put` and the struct header all got worse. v003 lost clearly on hit-rate and `put`, which is the falsifying outcome its hypothesis predicted.

## Patterns

- **Hit-rate depends on how many laps a key survives in main, not on how high its counter can go.** Halving bounds the drain cost, but it also removes the long linear retention that produced 0008 v003's gain. Together with 0008, this shows the depth lever is a trade between retention laps and drain work. A cheaper decay schedule does not separate them.
- **Direct ghost-to-main readmission holds the adaptive split together.** Ghost hits shrink small, and the readmitted key fills the slot main just gained. Routing ghost hits through small breaks that link: main grows empty while small starves toward its floor, and long-reuse keys bounce between small and ghost. Every probe of the ghost path has now lost hit-rate, whether it made readmission stronger (0002, 0005, 0008) or weaker (here). The current ghost behaviour looks like a local optimum.
- **Hash-table internals no longer limit `get`.** A leaner probe did not move `get` p50, and linear-probe misses plus backward-shift deletes moved cost onto `put`. The remaining `get` cost is the hash plus two dependent loads (the bucket, then `slots[i].value`). Earlier cycles found inlining the payload slow, so lowering that floor again needs something other than probe tuning.
- **Latency attribution is still unreliable.** v001 and v003 both saw `get` move even though their `get` code was effectively identical to the parent's. No variant had a same-run parent baseline.
- **`small_size`, `settled_small` and `floor_blocks` are still unreported.** This is the ninth cycle in a row. v003's mechanism (small pinned at the floor) is plausible but still unverified.

## Open questions

- Can 0008 v003's retention depth be kept with bounded eviction work, such as a fixed scan budget per eviction or decrementing a sampled subset, instead of a shorter retention schedule?
- Would a prefetch of `slots[i].value` issued from the probed bucket cut `get` p50 on hashbrown, without the costs of the inline layout?
- Is v001's `put`-tail gain real, or drift? Only a same-run re-baseline can tell.

## Continuation

Keep 0008 v001 as the parent, on hashbrown with decrement drain and direct ghost-to-main readmission. Drop halving decay, the custom probe table and all further ghost-readmission changes, in either direction. If the frequency-depth lever is pursued, keep the linear retention and bound the eviction work per `put` instead. Treat `get` as close to its floor unless a variant removes the second dependent load. The loop infrastructure must still run parent and child in the same benchmark and get the split-report stdout to the insight writers.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

This cycle had one depth-lever retry and two single-lever policy probes:

- **H01** went back to linear decrement on the 3-bit `MAX_FREQ = 7` counter and capped each `evict_main` at a fixed rotation budget of 32. Its parent was 0009 v001.
- **H02** reset a graduate's freq to 0 when it moves from small into main. Its parent was 0008 v001.
- **H03** lowered the upper clamp on `small_size` from `capacity/2` to `capacity/4`. Its parent was 0008 v001.

All three are Winners. v001 is the only one that moved hit-rate: it beat its parent and held `put` steady. It fell short of 0008 v003's level, and its `get` got worse even though the `get` code did not change. v003 and v002 left hit-rate exactly where it was, so both are null results that made the cut on rank.

## Patterns

- **Bounding the drain work separates retention depth from drain cost.** Cycle 0009 found that hit-rate comes from how many laps a key survives in main. Linear decrement plus a per-call rotation budget brings most of those laps back without the `put` tail that 0008 v003 paid. Unlike halving, the budget does not trade one for the other. Whatever gain is still missing likely comes from forced releases once the budget is spent.
- **Both bounds of the split are now ruled out as hit-rate levers on workload-a.** The 0005 floor did matter. The upper clamp, however, never binds: on workload-a, small appears to only shrink from its starting quarter, so the ceiling is dead code. This comes from v003's reasoning and is not yet confirmed by split-report data.
- **The credit a graduate brings from small doesn't matter.** Hot graduates get re-hit in main and saturate again before they reach the tail. Cold graduates' leftover laps are too few to change any eviction. Resetting freq at graduation gives the original S3-FIFO semantics at no hit-rate cost, but it is not a lever.
- **Latency drift went both ways in the same cohort.** v002 slowed on `get` and `put`, and v003 got faster tails. In both, the hot-path code was identical to the parent's. So v001's `get` regression can't be blamed on its eviction change without a same-run benchmark.
- **The split reports are still unreported.** This is the tenth cycle in a row. `forced_evictions`, `ceil_blocks`, `settled_small` and `main_rotations` were all added or requested, and none reached the writers.

## Open questions

- How often does the rotation budget bind? Would a larger budget (e.g. 64) close the rest of the gap to 0008 v003 without bringing back the `put` tail?
- Is v001's `get` regression real, perhaps from a different resident set changing probe or cache-line behaviour, or is it drift?
- Does small ever grow above its initial quarter on any workload? `ceil_blocks` and `max_small` from the Zipf sweeps would show it directly.

## Continuation

Adopt 0010 v001 as the parent: linear decrement, `MAX_FREQ = 7` packed into 3 bits, and a bounded `evict_main`. The next depth step is to tune the budget size, guided by `forced_evictions`. Drop all clamp tuning in either direction, and drop changes to graduate credit. v003 suggests the split's movement or the ghost size as the next lever, but triggers (0006, 0007) and every ghost probe (0002, 0005, 0008, 0009) have already failed, so treat that as low-priority. The loop infrastructure still has to benchmark each parent and child in the same run and get the `h01_split_report` stdout to the insight writers. No latency attribution in this cycle is trustworthy until it does.


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
