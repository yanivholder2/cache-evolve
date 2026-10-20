# Variant v002 Insight

## Helped

- hit-rate: `stash_insert_with_rescue` puts each stash victim back at the LRU end (`Front`) of its own target bucket whenever that bucket has a hole. Items the parent dropped while the normal tier still had free slots now stay resident, which uses the spare target capacity the parent left idle. The gain only just clears the hypothesis's threshold, so rescue's hit-rate benefit carries over to the tagged parent but does not grow there.
- metadata-size (held): the port is a single free function plus routing at the three stash-insert sites. `Item`, `Dash` and the parent's bucket header are untouched.

## Hurt

- per-op-time (get p50): the hypothesis is refuted. Get p50 lands at or above its own 78 ns refutation line, and the regression is about the same size as the one cycle 8 reported without tags. With tags, a fuller target costs only a few extra tag-byte compares per miss, so the regression cannot come mainly from walking more residents. The more likely cause is what fuller targets do to the rest of the `get` path. More stash hits now promote into a full target (`promote_to_target_h` → `demote_from_normal_bucket` → another stash insert, and possibly another rescue). Hits that stay in the target also rotate longer item and tag arrays in `get_from_position`.
- per-op-time (put p50/p99): every stash insert that evicts now costs an extra `hash(&victim.key)`, a probe of the target bucket's fill state (`is_full`) and possibly a tagged target insert (`put_at_h`, which also runs a tag-filtered "already present?" check). Demotion paths can trigger this up to twice per call: once for the demotee and once for the cascade evictee. The put tail grows much more than the median, which fits this extra work sitting on the eviction-heavy path.
- per-op-time (get p99): not improved. Multi-bucket miss chains cost about the same as in the parent, so the tags' tail benefit is preserved but not extended.

## Takeaway

Tags do not make target-only rescue cheap. They cut the cost of scanning a fuller bucket, yet the get-p50 regression stays about the same size as cycle 8's, so scanning was never most of rescue's cost. What remains is structural: keeping targets fuller sends more promotions into full buckets and drives cascade-and-rescue work onto both `get` and `put`. Rescue buys a marginal hit-rate gain at a latency price that tags cannot reduce. A cheaper next step would limit when rescue runs, for example only when the target is under a fill threshold, so promotion targets keep a hole. That would work better than trying to make the fuller buckets faster to probe.
