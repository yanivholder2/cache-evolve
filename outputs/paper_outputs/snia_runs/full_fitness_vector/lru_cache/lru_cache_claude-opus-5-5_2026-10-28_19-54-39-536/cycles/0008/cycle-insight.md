# Cycle 0008 Insight

## Cohort outcome

Two variants branched from cycle-0007 v002 (capacity/2 ghost at 131K) and made the ghost smaller. v001 narrowed the large-size ghost to capacity/4. v002 cut the ghost tags from 16 bits to 8. v003 branched from the ghost-free cycle-0004 v002 and widened probation to about 1/3, at 131K only. All three are Winners. v001 is the hit-rate gain: a small summary rise, plus timing gains that are probably partly noise. v002 is the clearest latency result so far: get p50 improved with bit-identical hit-rate. v003 is a weak Winner: neutral to slightly negative on the summary, and most useful as a noise-floor and negative control.

## Patterns

- The ghost table's byte footprint slows get, even though get never reads the table. Both variants that shrank the ghost (v001 fewer slots, v002 narrower tags) had lower per-op times on unchanged get code. v002 changed only the tag width, so it is the first variant to isolate a cause of the long-running get-p50 regression. The footprint suspect from cycles 6–7 is now the leading explanation, not a guess.
- At 131K, hm_0 and web_0 pull in opposite directions on any probation-time lever. A narrower ghost (v001) and a static wider probation (v003) both traded hits between the two traces. Only the ghost-targeted version came out net positive. This repeats cycle 5's result at 16K: selective reprieves beat a uniformly wider window at both sizes.
- Ghost width tuning is nearly used up. capacity/4 is slightly better than capacity/2, and the moves on both traces are now marginal.
- The noise floor is now visible. v003 left the hot paths identical, yet its per-op times moved noticeably. Any per-op delta of similar size in v001 or v002 should not count as evidence on its own. v002's get-p99 rise has no code cause and fits this noise.
- v002's hit-rate was bit-identical even though 8-bit tags collide far more often. This is plausible, because spurious reprieves are cheap. It could also mean the harness never exercises those collisions. This has not been checked.

## Open questions

- Do v001 and v002 stack? A capacity/4 ghost with 8-bit tags would be a quarter of the parent's ghost bytes at 131K. Does get p50 keep falling?
- Can the ghost get smaller still, for example with 4-bit tags, a bitset or a Bloom-style ghost, before the hit-rate cost becomes visible?
- Are spurious reprieves actually happening in the hit-rate harness? Count them.
- Is v002's get-p50 gain larger than v003's noise swing when measured in repeated paired runs?

## Continuation

Make the base cycle-0008 v001 combined with v002's 8-bit tags (capacity/4 ghost, u8 tags). Keep shrinking ghost bytes as the latency lever, and add a spurious-reprieve counter to confirm hit-rate neutrality. Drop static probation/protected split retuning at every size, and drop further ghost-width tuning. Any new hit-rate lever at 131K should change how reprieves are granted, not the window size. Report get-p50 claims against v003's noise floor and confirm them with paired runs.
