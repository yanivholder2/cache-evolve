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
