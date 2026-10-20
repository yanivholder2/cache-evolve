# Cycle 0008 Insight

## Cohort outcome

Each variant was on the CLOCK line and tested one lever. v001 keyed the map by the precomputed hash so each key is hashed once. v002 doubled the sketch width again. v003 put a periodically cleared doorkeeper in front of the sketch. All three are listed as Winners, but none moved a headline axis forward. v001 kept hit rate identical but slowed down a little. v002 gained a hit-rate amount no bigger than noise and paid for it in median latency. v003 lowered hit rate and raised latency.

## Patterns

- Hashing is not the cost. Cycle 0006 removed a duplicate hash and this cycle removed the map-side re-hashes, and neither changed latency. For `u64` keys the Fx-like hash is nearly free, and any guard compare or extra branch on the hit path costs more than it saves. Drop hash-count reduction as a latency lever.
- The sketch's memory footprint is now the clearest per-op cost. This is the first time a median regression has a clear mechanism behind it: the doubled table no longer sits comfortably in L1/L2, so `get` p50 went up even though `get` code was untouched. Each access makes four scattered counter writes, and that is where the `get` time goes.
- Sketch-input levers are used up from both directions. More width buys nothing measurable. Filtering first sightings (the doorkeeper) hurt, because a small filter fills with one-hit keys and its all-at-once clear acts as abrupt aging. That matches the earlier halving and aging failures. The strict frequency duel with non-decaying counters still wins on workload-a.
- Refactors that preserve behaviour still keep hit rate exactly (v001), so the CLOCK base stays safe to change. The p99 readings still move in both directions with no explanation in the diffs.

## Open questions

- Would a smaller or denser sketch (narrower counters, rows packed into one cache line per key, or fewer rows) cut `get` latency without losing hit rate? This is the reverse of this cycle's widening.
- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU on CLOCK? Every admission, aging, window and sketch-accuracy change since the early cycles has been flat or negative.

## Continuation

Keep cycle 0006's v001 (CLOCK, width factor 8) as the base, and adopt none of this cycle's variants. Drop the following:
- hash-once and identity-keyed maps
- further sketch widening
- doorkeepers and any periodic or abrupt aging

If latency is the target, cut the sketch's memory traffic on each access (for example, rows blocked into one cache line, or a smaller footprint). Require hit rate to stay identical or within noise, and judge by medians repeated side by side against the parent. Otherwise, treat hit rate as near its ceiling and prefer behaviour-preserving changes.
