# Cycle 0009 Insight

## Cohort outcome

This cycle probed two further extensions of geometry (pushing the normal/stash split past 7/8 toward near-elimination of the stash) and one counter-accounting fix (crediting a stash-hit access on the `lfu_counter` before target insertion). Both geometry extensions regressed or were neutral; only the counter-credit change won, delivering a modest positive lift.

## Patterns

- **Geometry has reached its effective ceiling at 7/8.** Widening the normal tier further to 15/16 reversed the gain trend: with only two stash slots remaining, cold-arrival bursts that the parent's four-slot stash absorbed without harm now compete directly with hot-tier residents, and second-chance recovery cannot compensate for the lost headroom. The inflection point is clearly between 28 and 30 normal buckets for this workload.
- **The stricter promotion gate is ineffective at the 7/8 geometry.** Requiring two in-tier accesses before promotion targets a loss site — spurious single-hit promotions competing for scarce probing slots — that only exists when the normal tier is narrow. At 7/8 width, items reaching the promotion boundary are already strongly hot; the gate only delays correct promotions without reclaiming meaningful capacity.
- **Pre-insertion counter credit at the stash exit closed a real but modest protection gap.** Without this change, stash-demoted items re-entering the target tier carry a zeroed `lfu_counter`, making them indistinguishable from cold newcomers under `ClassicLRUFreqProtect` and immediately eviction-vulnerable despite having just been accessed. Crediting the access before insertion eliminates that one-access window. The win is genuine but smaller than prior geometry gains, indicating the vulnerability window existed but was not the dominant loss site.

## Open questions

- With geometry exhausted and the stash-exit protection gap now closed, what is the next meaningful loss site — `lfu_counter` staleness for long-lived hot items, or some structural asymmetry in how the probing bucket handles demotions?
- Could a soft cap on `lfu_counter` prevent established hot items from accumulating stale high counts that crowd out newly warm items, and would that interact positively with the stash-exit credit now in the baseline?
- Does the stash-exit counter credit compound with any remaining counter-management axes, or does it represent the last per-item accounting gain available at this geometry?

## Continuation

The 7/8 geometry is confirmed as the ceiling; the next hypotheses-writer should treat it as fixed and not revisit geometry expansion. v003's stash-exit counter credit is the new champion and should be the baseline going forward. The most promising remaining axis, flagged by multiple prior cycles, is soft-capping `lfu_counter` to prevent staleness in long-lived hot items — this is the primary thread to pursue next. The stricter promotion gate (two-hit requirement) is effectively exhausted at this tier width and should be deprioritised unless paired with a geometry change that makes probing slots scarce again.
