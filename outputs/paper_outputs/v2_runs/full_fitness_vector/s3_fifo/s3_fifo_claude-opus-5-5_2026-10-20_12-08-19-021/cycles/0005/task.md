# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
