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
