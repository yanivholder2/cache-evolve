# Variant v002 Insight

## Helped

- per-op-time: `get` no longer runs `unlink` + `push_back` on a hit. It only sets one `referenced` bit on the node it already found, so it no longer touches neighbouring arena nodes. That brought down both median and tail `get` latency, and the tail fell the most.
- per-op-time: `put` also got faster even though it was not the target. Updating an existing key is now an in-place write plus a bit-set. Evicting overwrites the slot under the clock hand instead of relinking the tail and head. On this trace the sweep usually stopped quickly, so the extra cost of scanning past referenced slots never showed up in the `put` tail.
- metadata-size: removing `prev`/`next` from `Node` takes out two `usize` fields. The new `referenced: bool` fits in what would otherwise be padding. Dropping `head`/`tail` for a single `hand` likewise shrinks the `LruCache<u64,u64>` struct.
- hit-rate: the hypothesis expected CLOCK to lose a little hit rate against strict LRU, but it gained some on workload-a at size 2048. CLOCK's one-bit second chance kept useful entries that strict LRU recency order evicted on this access pattern.

## Hurt

(None. No dimension got worse compared with the baseline.)

## Takeaway

Most of the baseline's cost was in reordering the recency list on every hit, and LRU's exact order wasn't needed for hit rate here. Swapping it for a CLOCK reference bit improved all three dimensions at once, so the expected hit-rate trade-off never happened on this trace. That hit-rate result comes from a single trace at a single cache size. The sweep in `put` is now the only part whose cost depends on the workload, so later variants should check the `put` tail on workloads where most of the ring is referenced at the same time.
