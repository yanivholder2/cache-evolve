# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0003/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
