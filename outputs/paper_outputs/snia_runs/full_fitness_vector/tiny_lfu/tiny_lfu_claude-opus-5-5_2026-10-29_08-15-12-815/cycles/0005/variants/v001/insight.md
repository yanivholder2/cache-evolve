# Variant v001 Insight

## Helped

- per-op-time: Get p50 dropped to about where the 40-byte-node line sits, and get p99, put p50 and put p99 all fell sharply too. Several changes land together. All link arithmetic now stays in `u32` with direct `NIL` compares, so the `usize` widen/narrow round-trips are gone. `push_back` writes `prev` as one tagged store (`(r << REGION_SHIFT) | tail`) where it used to do two dependent mask/or read-modify-writes. `unlink` no longer resets the detached node's links. Every link-path access goes through `get_unchecked` (`node`/`node_mut`), and `lists` is indexed by `r & 3` over a 4-slot array. The sketch is byte-identical to the parent, so the whole latency recovery comes from residency-layer codegen.
- metadata-size (`TinyLfuCache<u64,u64>` struct component): The cache struct got smaller. `RegionList`'s head/tail/len fields shrank from word-sized to `u32`, which more than covers the unused fourth `lists` slot added so the region-tag indexing folds its bounds check away. The headline `Node<u64,u64>` figure is still 24 bytes, so the summary value did not move.

## Hurt

(none) hit-rate is identical in every trace/size cell. That is the expected result: admission, eviction and the sketch are logically unchanged, and the new 10k-op list-consistency test guards the link rewrite.

## Takeaway

Cycle 0004 concluded that the 24-byte line's latency penalty came from u32-link codegen (bounds checks plus mask/widen sequences in `unlink`/`push_back`/`on_hit`), not from the smaller node layout. This variant confirms it. Keeping indices in `u32` end to end, collapsing the tagged-`prev` update into one store, and removing bounds checks in the link paths keeps the 24-byte metadata win and takes away its latency cost. Put latency improved even more than get, which fits put running more link operations per call (eviction contest plus window/probation migration). The change is not attributed per step: the hypothesis landed all of these edits together.
