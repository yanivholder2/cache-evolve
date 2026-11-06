# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle-0004 v002 (single-`Entry`, no-clone SLRU). v001 ported the fingerprint reprieve with its flag in `prev` and the ghost enabled only for small caches. v002 replaced protected-hit relinks with a CLOCK reference bit. v003 used a static 1/3 probation share at small sizes. v001 is the only Winner. It lifted both traces at 16K and matched the parent at 131K, but gave back v002's put gain by doubling hash work, and get p50 still regressed.

## Patterns

- Gating by size works for hit-rate mechanisms. Both v001 and v003 left 131K untouched by design, so every hit-rate change came from 16K alone. v001's gate removed the large-size hm_0 loss from cycle 4.
- At 16K the reprieve beats resizing. A static wider probation helped hm_0 but hurt web_0, so no fixed share suits both traces. The reprieve gives extra probation time only to keys that come back, and it helped both. Selective second chances do better than a uniformly bigger window.
- Put cost still tracks hash count. v001's separate `hash_one` for ghost tags, on top of the map's own hashing, erased the single-probe gain. This is the third cycle confirming the rule.
- Hit-path changes still hurt get p50, even tiny ones. An extra AND in `set_protected` (v001) and CLOCK moving list work into promotion sweeps (v002) both regressed it. Removing relinks from protected hits did not lower the median. That is more evidence that get p50 is not driven by list work.
- The timing benchmark is too noisy to rank small per-op changes. v003 changed nothing on the hot path, yet its per-op times swung widely in opposite directions. Get p99 swings in every variant now look like noise, including cycle 4's p99 rise.

## Open questions

- Can v001's ghost reuse the hash the map already computes (raw-entry or `hashbrown` API), and run only on the vacant path, to restore v002's put cost?
- If `set_protected(true)` is kept to a single OR and `SC_BIT` is cleared at reprieve or eviction time, does get p50 recover?
- How much timing movement is just run-to-run variance? Repeated paired runs of parent and child are needed before trusting per-op deltas under a few percent.

## Continuation

Make v001 (size-gated `prev`-bit reprieve) the base, and fix its costs next. Derive the ghost tag from the map's own hash, skip ghost work on occupied-key puts, and make `set_protected` a single OR. Drop protected-segment CLOCK and static probation-share retuning. Judge latency claims only from repeated paired runs.
