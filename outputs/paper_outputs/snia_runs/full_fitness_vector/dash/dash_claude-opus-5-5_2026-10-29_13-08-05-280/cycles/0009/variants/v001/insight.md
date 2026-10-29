# Variant v001 Insight

## Helped

(none: this was planned as a no-change noise control, and nothing improved)

## Hurt

- per-op-time: The intended change was a single comment line with no behavioural effect, yet every latency metric got much slower. That covers get p50/p99 and also put p50/p99, and `put` code has been untouched since the parent's lineage. Get p50 is far outside the ±5 ns band the hypothesis set for "confirmed", and the tails moved more than the medians. A shift this large and this uniform on a binary that should be the same can only come from the environment (host or session state, frequency scaling, co-tenancy, cache/TLB warmth). It does not come from the code.

## Unchanged (validity check)

- hit-rate: All four cells (web_0 and hm_0, 16384 and 131072) match the parent exactly. That is consistent with the copy being functionally identical, which is what H01 needs for the variant to count as a valid control.
- metadata-size: Item and Dash sizes are the same as the parent's. Layout was not disturbed.

## Takeaway

H01 is **refuted**. Re-running the cycle 0007 v003 incumbent with only a comment added produces a large uniform slowdown across both get and put. Cross-session per-op-time comparisons in this run are therefore dominated by environmental drift and cannot support latency verdicts. That includes cycle 0008 v001's apparent get regression. This session's get latency for the incumbent copy is in the same range as that variant's, so the regression may be mostly or entirely drift. From now on, latency should be scored only against a same-session control like this one: compare H02/H03 to this v001's per-op-time, not to the numbers recorded for the parent. Caveat: I couldn't list or open the overlay source in this session. That the binary is identical is inferred from the exact hit-rate match and from the hypothesis, not from a diff.
