# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle 0001's v001 (single-reuse graduation). v001 gated the looser threshold on main having room. v002 made the small/main split adaptive, ARC-style. v003 combined the looser threshold with a static 15% small queue. v001 won mostly by undoing its parent's change: main stays full once the cache warms up, so the gate falls back to the baseline two-reuse rule.

## Patterns

- Which keys get into main matters more than queue sizes. Both split variants moved hit-rate by close to noise, and their effect flipped sign across traces and cache sizes. The only clear gain came from stricter admission into a full main.
- Under steady-state pressure, the one-reuse threshold costs hit-rate on web_0 and also on hm_0 at the large size. Its benefit holds only on hm_0 at the small size. Cycle 0001's "hm_0 wants a looser threshold" result was mostly a cold-start or small-cache effect.
- Main occupancy doesn't work as an adaptive signal: it fills and stays full. Any adaptive gate needs a signal that keeps changing in steady state, such as the ghost hit rate or the small queue's reuse rate.
- An unweighted ghost-hit controller probably drifts toward a larger small queue, because the two ghosts differ in size. That brings back the exactly-twice pollution. It also adds a second linear ghost scan on every miss.
- Per-op-time and metadata-size are still unmeasured. Cycle 0001's request to re-measure the hash-index refactor wasn't followed, since all of this cycle's variants descended from the policy line.

## Open questions

- Can a steady-state signal (ghost hit rate, small-queue reuse rate) usefully switch the threshold between 1 and 2, or is the baseline rule simply right for these traces?
- Does the adaptive split move at all? Where `small_size` settles on each trace was never logged.
- How much per-op-time does the extra `main_ghost` scan cost, and how much does the O(1) refactor save?

## Continuation

Treat v001 (essentially the baseline two-reuse rule plus freq reset) as the policy base. Stop tuning the static split, and don't adopt the unweighted adaptive split. Spend one hypothesis on per-op-time and metadata-size, ideally moving the policy onto the O(1) index/slab refactor. If policy work continues, gate admission into main on a signal that keeps changing in steady state, not on main occupancy. A size-weighted ARC step that also logs where `small_size` settles is a secondary option.
