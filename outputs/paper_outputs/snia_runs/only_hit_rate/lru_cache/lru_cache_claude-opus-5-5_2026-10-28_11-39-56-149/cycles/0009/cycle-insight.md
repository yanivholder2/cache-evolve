# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008's v002, and each was size-gated. v001 added a slow ±2 hill climber on the protected share at large size. v002 doubled the sketch's `reset_at` to 32 × capacity at small size. v003 picked the colder of the probation LRU and the protected LRU as the main-area victim at large size. v002 is the Winner. It is the first gain on small-size web_0 in many cycles, and it also lifted small-size hm_0 at no cost. Both large-size adaptive levers made things worse.

## Patterns

- The small-size limit was the sketch's memory length. Small-size web_0 resisted climber, split, admission and doorkeeper changes, but it moved once frequency history survived across its multi-fill reuse gaps. This is the first lever that helped both small traces together and did not trade one against the other. It agrees with the earlier finding that faster aging hurts: at tight capacity, longer memory helps on both counts.
- Size gates held again in every variant. Cells across each gate were exactly unchanged, so each result can be attributed to its one lever.
- The large-size adaptive levers failed because their signals were weak or biased, not because adapting is a bad idea. The split climber could not tell a 2-point split change apart from web_0's phase changes and the window climber's moves. Shrinking applies at once while growing fills in lazily, which biased it toward a smaller protected share. The frequency-based victim rule compared decayed protected counts against probation counts that had just won the duel, so in effect it evicted protected first. That removed the long-reuse residency that large-size web_0 relies on.
- Protected residency is still load-bearing at large size for both traces, not only for hm_0. Any rule that drains protected faster hurts web_0 as well.
- For the ninth cycle, only hit-rate was measured.

## Open questions

- How far does the small-size `reset_at` gain extend (48–64 ×) before stale counts start to block new hot keys?
- Would a longer aging period also help at large size? It could help keep large-size web_0's long-reuse keys warm, and it is untested.
- Could ghost hits or a shadow comparison give a clean enough signal to adapt the split at large size?

## Continuation

Use cycle 0009 v002 as the new parent, with 32 × `reset_at` at small size. Next, push the small-size multiplier further (48 × and 64 ×), and separately try a longer `reset_at` at large size only. Both are low-risk and use one lever each. Drop per-eviction frequency victim selection and non-decaying hit-rate climbers on the protected split. Only revisit split adaptation with a ghost-hit or shadow signal. `win_cap`, `metadata-size` and `per-op-time` are now nine cycles overdue.
