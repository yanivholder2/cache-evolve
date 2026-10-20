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
