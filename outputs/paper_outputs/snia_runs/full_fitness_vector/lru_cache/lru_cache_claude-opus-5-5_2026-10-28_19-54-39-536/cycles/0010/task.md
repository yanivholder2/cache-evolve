# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two variants targeted latency on the cycle-0004 v002 base without changing policy. v001 split eviction and promotion into cold functions and forced the hot helpers inline. v003 removed the victim before inserting the new key on stock hashbrown. v002 turned the cycle-0006 v001 reprieve on at 131K with a capacity/2 ghost. The Winners are v001 and v002. v002 is the clean gain: web_0 at 131K improved more than hm_0 dipped, and 16K was unchanged. v001 won only on the get tail, and its get p50 got worse.

## Patterns

- Two unrelated changes both "fixed" cycle 4's get-p99 rise. One was a pure layout split (v001), the other a change in eviction order (v003). Both also made get p50 worse. When different causes produce the same effect, noise or broad layout sensitivity is a more likely explanation than any one mechanism. Neither claimed cause has been established.
- Get p50 still resists every lever. Forced inlining made it worse, and v003 changed no get-path instruction and it still rose. Under a p50 summary, trading the median for the tail has now lost in two variants.
- Put cost tracks hash count, now for the fifth cycle. v003's extra SipHash on every evicting put undid the single-lookup saving.
- The ghost window works as a reuse-distance filter. Cycle 6 showed capacity/8 is too short for web_0 at 131K. capacity/2 catches most of web_0's useful re-references and lets in only a few of hm_0's loop keys. The best width probably lies between the two. With the zero-seeded Fx ghost, hit-rate deltas can now be trusted.
- `metadata-size` does not count the heap ghost table. At 131K the ghost is a large hidden memory cost.
- Two of the three variant insights were written without access to the overlay source. Their explanations assume the hypothesis was implemented as written.

## Open questions

- Is capacity/4 (or stricter reprieves at large sizes) a better point on the web_0-gain/hm_0-leak trade-off than capacity/2?
- Does v001's `#[cold]` split without the `#[inline(always)]` hints keep the tail fix without the get-p50 loss?
- Is any of this cycle's get-p50 or get-p99 movement real? None of it has been confirmed by repeated paired runs.

## Continuation

Use cycle-0007 v002 (capacity/2 ghost at 131K) as the new base. Tune the large-size ghost width toward capacity/4 or tighten large-size reprieves. Consider counting the ghost table in metadata. Drop forced `#[inline(always)]` and drop remove-before-insert using a second hash. Retry an eviction-order change only through a hash-preserving path (`HashTable`/raw entry). Stop chasing get p99 or p50 until paired repeated runs show the effect is larger than noise.


### Cycle 0008

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Two variants made the ghost smaller and one replaced it. v001 stacked capacity/4 with 8-bit tags. v003 replaced cycle-0008 v002's tag table with a 1-bit bitset aged by a rotating cursor. v002 dropped the ghost and gave demoted-from-protected keys a reprieve marked by a flag bit. All three are Winners, but only v001 is a clean gain: get and put were both faster, at a cost of the smallest visible hit-rate loss. v003 kept hit-rate roughly level but made get p50 and put p50 worse. v002 lost hit-rate and did not move latency in any interpretable way.

## Patterns

- The ghost-footprint theory of get p50 is now in doubt. At 131K, v001 and v003 have the same ghost bytes, and get never touches the ghost in either. Yet v001's get p50 fell and v003's rose. v003's ghost is even smaller at 16K. So footprint alone cannot explain the get-p50 moves. Noise, or layout or codegen effects from changes to the header and to `put`, are at least as likely. Cycle 8's "isolated cause" should be downgraded to "unconfirmed".
- Get p50 again moved without any change to the get path. v002 rose with no new get branch, and v003 rose with byte-identical get code. This is the same noise or layout sensitivity seen in cycles 6–8. No single-run get-p50 delta of this size counts as evidence.
- Put cost tracks work in the `Vacant` arm. A denser table made put cheaper (v001). Cursor bookkeeping and read-modify-write bit operations made it more expensive (v003), even with fewer bytes. This continues the multi-cycle "put cost = hash/bookkeeping count" pattern.
- Ghost collisions now show up in hit-rate. v001 (8-bit tags) and v003 (tagless bits) both moved hit-rate on real traces. So cycle 8's bit-identical result was luck at that configuration, not proof that the harness never runs the ghost path. The hit-rate cost of shrinking the ghost is real but marginal at both sizes.
- Reprieves need evidence gathered after eviction. A flag for "was once protected" (v002) predicts reuse poorly: it keeps cold keys at the expense of newcomers, especially in the small probation at 16K. Only a ghost hit, meaning a re-reference after eviction, has paid off in any cycle.

## Open questions

- Is v001's get-p50 gain larger than the noise? Run paired, repeated measurements of v001, its parent, and v003 to decide whether footprint matters at all.
- At 131K, does the ghost still pay for its bytes and put cost compared with no ghost at all, given its short window there?
- Can the bitset ghost keep its hit-rate without paying the per-put aging cost, for example with a generation flip on a double buffer instead of a word sweep?

## Continuation

Use cycle-0009 v001 (capacity/4, 8-bit tags) as the base. Before claiming any more get-p50 effects, establish a measured noise floor using paired repeated runs. Drop flag-only or pre-eviction-history reprieves. Drop further ghost shrinking as a get-latency lever: judge ghost changes only on hit-rate and put cost. If a bitset is retried, aging must add no per-put work. Any new hit-rate lever should change how ghost-hit reprieves are granted, not the window size.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
