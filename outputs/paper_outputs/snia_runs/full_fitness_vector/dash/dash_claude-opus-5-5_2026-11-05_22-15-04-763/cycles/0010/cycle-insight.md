# Cycle 0010 Insight

## Cohort outcome

All three variants carried out 0009's plan. v001 merged a per-bit collision word with the k=2 mask (on 0009 v002). v002 moved the stash-promotion chain into a cold, out-of-line function that uses slice rotations (on 0009 v002). v003 widened the stash victim window to the whole bucket with a protect-once bound (on 0009 v003). All three are Winners only as members of the cohort. v002 is the only one that hit its target, a clear `get` p99 drop, and it paid for it in `put` p99. v001 and v003 regressed with no offsetting gain.

## Patterns

- Every variant moved cost into the `put` tail. A collide word on the write path (v001), shared helpers made cold (v002), and more frequent mid-bucket evictions (v003) each landed on `put` p99. For three cycles now, any extra work or layout change on the stash insert/eviction path shows up in `put` p99 first.
- Under k=2, collision tracking is a net loss. With a full 64-bit word and about 8 keys per bucket, collisions are common. Stale bits create more collide marks, so the lazy rebuild fires on a steady schedule and costs more per rebuild. A second word on the hot path costs more than the refresh it was meant to avoid. The k=1 O(1) clear (0009 v001) and the k=2 mapping do not stack this way.
- The `get` tail hypothesis is now confirmed by direct intervention: making the promotion chain cheap and out of line cut `get` p99. But that chain shares code with `put`'s full-bucket insert/demotion, which is routine for writes. Marking the shared code cold moved the tail from reads to writes. The i-cache effect on `get` p50 was marginal at best.
- Stash victim selection is settled as a dead lever. The narrow and the whole-bucket windows both leave hit rate on the same plateau. The wide window slightly hurts hit rate because it evicts young stash entrants before their first hit. The hit-rate ceiling is not set by which stash slot gets evicted.
- No variant could inspect its segment source. The mechanism claims (cache-line layout, whether `put` goes through the cold path, whether rotate compiles to a memmove call) are inferred from deltas, and the tail moves rest on a single sample.

## Open questions

- Does `rotate_right` on short fixed-size buckets compile to an out-of-line memmove that costs more than an unrolled loop, and is that the `put` regression rather than the cold call itself?
- If `get` keeps a cold promotion entry and `put` gets its own inlined shift, does v002's `get` p99 win survive without the `put` p99 loss?
- With victim choice ruled out, what does set the hit-rate plateau? Stash size versus normal-tier capacity, the admission and demotion rules, or the two-hit promotion gate?

## Continuation

Build on 0010 v002 and split its paths. Keep the cold out-of-line promotion for `get` only, and give `put`'s full-bucket insert/demotion its own inlined shift. Also try that variant with an unrolled loop in place of `rotate`. Drop collision-word O(1) clearing under k=2, and drop stash victim selection entirely. For hit rate, try a different axis, such as the promotion gate or stash/normal capacity split, rather than eviction choice. Rerun any tail-only result before building on it.
