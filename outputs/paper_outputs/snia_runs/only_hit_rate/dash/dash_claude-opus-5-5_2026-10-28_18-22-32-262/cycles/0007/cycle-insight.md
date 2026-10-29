# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle-0006 v001, which combines the ghost ring with `Back` probing placement. v001 sent displaced foreign residents straight to the stash when their target was full. v002 rescued stash-overflow victims into free target or probing slots. v003 halved the ghost ring to factor 1. The Winner is v002. It gained on `hm_0` 128K, was marginally up on `web_0` 128K, and regressed nowhere. Its target cell, `web_0` 16K, stayed flat. v001 matched the parent exactly. v003 was mixed: a slight gain on `web_0`, a slip on `hm_0` 128K.

## Patterns

- The `web_0` 128K giveback is still unexplained. The foreign-retry cascade is now ruled out: v001 had no effect at all, which suggests the retry already evicts the foreign item itself. The rescue (v002) and the shorter ring (v003) moved it only marginally.
- Levers that use free capacity only pay off at the large size. At 16K, normal buckets are saturated whenever the stash overflows, so v002's rescue rarely fires there. `web_0` 16K is a capacity-saturated cell, not an idle-slot or ghost-reach problem.
- Ghost reach has an optimum that depends on the trace. `web_0` prefers the shorter ring, which gives less stale credit. `hm_0` 128K needs the longer tail, because its keys return between one and two stash-capacities after eviction. Most of the ghost's value comes from returns within one stash-capacity. Factor 2 remains the safer global setting.
- No-cascade, free-slot-only moves stay safe: v002 displaced no residents and regressed nowhere. This fits the lineage's lesson that an item that has lost its slot must not outrank residents.
- Per-op-time and metadata-size are still unmeasured, for the seventh cycle in a row. v003's cost argument is unverified as a result.

## Open questions

- At 16K, does a different victim choice in full normal buckets move `web_0`? Candidates are recency-vs-frequency victim selection and ghost-informed victim choice.
- Could an adaptive ghost reach (per trace, or occupancy-conditioned) combine v003's `web_0` gain with factor 2's `hm_0` 128K tail?
- Does v002's rescue push targets below the bypass free-slot threshold and silently turn bypass promotions into gated ones?

## Continuation

Build on v002. Drop foreign-retry changes and further fixed ghost-factor tuning. Treat the rescue as settled for the large size. For `web_0` 16K, change which resident is evicted from a full bucket rather than looking for empty slots. Investigate the `web_0` 128K giveback through ghost churn from probing evictions. Measure per-op-time and metadata-size.
