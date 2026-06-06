# Variant v002 Insight

## Helped

- hit-rate: The bounded ghost ring records the one place a key truly leaves the cache (the `stash_insert` drop branch) and, on that key's next `put`, fast-tracks it via `promote_to_target(..Back, Back)` straight into protected-target MRU — converting an "admitted, dropped, re-requested" event into a LIRS-style short-reuse-distance signal that earns a handful of recurring keys residency the parent's stash-only re-admission would have made them re-prove from scratch.

## Hurt

- (no dimension regressed) Keeping the cold path unchanged for never-ghosted keys — routing first-touch admissions through `stash_insert` and gating the fast-track strictly on `ghost.contains` — avoids the dwell-time loss that front-inserting *all* cold keys caused, so genuine first-touch keys keep their stash dwell and the keys-only ring leaves capacity untouched.

## Takeaway

Unlike the parent's stash relocation, which chased the same warm-key pool the protected tier already held, the ghost-list gate targets a population promotion never recaptures: keys that were fully evicted and then re-requested within a bounded window. That orthogonality is why it nets a small positive on top of the winner rather than washing out — but the gain's small magnitude says the dropped-then-recurring population is thin in workload-a, so the reuse-distance signal is real but narrow.
