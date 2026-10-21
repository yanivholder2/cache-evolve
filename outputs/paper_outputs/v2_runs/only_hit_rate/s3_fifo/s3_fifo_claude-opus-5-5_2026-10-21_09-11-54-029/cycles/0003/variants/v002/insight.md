# Variant v002 Insight

## Helped

(No measured dimension moved. As designed, hit-rate on workload-a matches parent v003 exactly. That is the expected result of a behaviour-neutral port and does not count as an improvement.)

## Hurt

(No measured dimension regressed. The port keeps all of v003's policy constants and control flow. These are the 5% small split, `MAX_FREQ = 3`, `freq > 1` graduation, ghost hits pushed to the front of main with freq 0, and the second-chance decrement loop in `evict_main`. Only the containers changed: a slab plus `index: HashMap<K, usize>` for residents, and a lazily deleted `(K, seq)` ghost deque backed by `ghost_set`. The eviction sequence therefore has nothing to change hit-rate.)

## Takeaway

The identical hit-rate is evidence that the port is exact. It agrees with the in-module `matches_v003_reference_on_pseudo_random_streams` test, which replays streams through a verbatim copy of v003 and compares the final small, main and ghost contents. The port is exact because of an invariant that v003's `ghost.retain` relied on without saying so: a key enters the ghost only when evicted from small, and a ghost hit sends it to main, which never re-ghosts. Each key therefore appears in the ghost at most once. That invariant is why a single-seq `ghost_set` with "count only live entries" capacity (`ghost_set.len() >= main_size`) reproduces v003's ghost bound exactly.

The point of this variant is the cost of each operation, and the fitness vector does not show it. `per-op-time` was again not measured. The expected gain is replacing an O(capacity) resident scan per `get` and an O(main_size) ghost scan per `put` miss with O(1) hashing. That gain is still unconfirmed, and it needs a timing dimension before this variant can be preferred over v003. One trade-off to watch is that the slab, the index and the duplicated ghost keys (deque plus map, with up to roughly 2×`main_size` stale deque slots before compaction) increase metadata memory, which would hurt `metadata-size` if that is measured. With this structure in place, future policy changes that enlarge the ghost or main queue no longer make lookups slower.
