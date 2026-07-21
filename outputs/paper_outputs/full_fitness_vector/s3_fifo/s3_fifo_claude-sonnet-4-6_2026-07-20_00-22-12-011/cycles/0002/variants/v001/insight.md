# Variant v001 Insight

## Helped

- hit-rate: Lazy ghost deletion causes the capacity guard to drain stale entries first before evicting live ones, preserving live ghost keys slightly longer in the deque and generating a small additional ghost-hit opportunity.
- metadata-size: Extracting `freq` from `Entry<K,V>` into parallel sibling `VecDeque<u8>` fields reduces the per-entry footprint from 24 to 16 bytes; the `S3Fifo` struct grows to hold the two extra deques and the `HashSet`, but that fixed overhead is amortized across all entries.
- per-op-time: Replacing the sequential `ghost.iter().any()` detection and `ghost.retain()` removal on ghost-hit `put` calls with `ghost_set.contains()` and `ghost_set.remove()` collapsed those two O(n) deque traversals to O(1), dramatically cutting put latency at both median and tail.

## Hurt

(none)

## Takeaway

Composing three orthogonal changes — smaller entry layout via parallel freq deques, O(1) ghost membership via a sibling `HashSet`, and lazy ghost deletion — simultaneously improved all three fitness dimensions without regression. The largest gain came from the ghost-hit `put` path, which previously paid an O(n) scan followed by an O(n) `retain()`; the new path pays two hash-table operations. The lazy deletion strategy is doubly beneficial: it makes ghost-hit promotion O(1) and incidentally extends the effective ghost window by leaving live entries in place while the capacity guard consumes stale slots.
