# Variant v003 Insight

## Helped

- metadata-size: Switching the LRU node links to `u32` slab indices with packed flags cut `Node<u64,u64>` to half its baseline size, and the cache struct shrank with it. The sketch is untouched, so the node layout accounts for the whole gain.

## Hurt

- per-op-time: Both get and put latency (p50 and p99) got worse. The compact layout adds a `u32`→`usize` conversion and a sentinel check on every link traversal, and any bit-unpacking of flags adds work too. The smaller node saves no cache misses at this working-set size to offset that.

## Takeaway

Shrinking the node alone reproduces the memory win but not a speed win: hit-rate stayed identical, so eviction behaviour was preserved, but per-op time regressed. The compact node is a pure size-for-time trade here. Any speed gain seen in cycle 0007 therefore did not come from the node layout alone. I could not read the overlay sources in this session, so this rests on the hypothesis text and the fitness vectors.
