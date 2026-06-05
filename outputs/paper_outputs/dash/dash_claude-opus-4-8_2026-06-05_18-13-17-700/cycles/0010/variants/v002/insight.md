# Variant v002 Insight

## Helped

- (no dimension moved) Grading the ghost ring by hit count — incrementing in `Ghost::record` instead of the parent's early-return no-op, and routing a count≥2 key into probing MRU via `promote_to_probing` instead of target — neither helped nor hurt hit-rate; it landed exactly on the parent.

## Hurt

- (no dimension regressed) The keys-only `HashMap<K,u8>` + `VecDeque` ring still adds no resident seats, the cold first-touch path through `stash_insert` is untouched, and `promote_to_probing` conserves occupancy, so `capacity_conservation` and `single_eviction_per_put` invariants held and hit-rate did not fall.

## Takeaway

Promoting twice-ghosted keys one tier higher into probing produced zero hit-rate change on workload-a, which means the persistently-recurring population the hypothesis bet on — keys dropped from the cache and re-requested *more than once* within the bounded ghost window — is effectively empty here: almost every ghost hit carries count 1 and takes the unchanged target branch. The graduation is sound but finds nothing to grade, confirming the parent's rationale that the dropped-then-recurring signal is thin; deeper recurrence isn't where the remaining hit-rate on this trace lives.
