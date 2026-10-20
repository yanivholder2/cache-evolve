# Variant v001 Insight

## Helped

- per-op-time: `push_front` costs the same as the `push_back` it replaces, and the bimodal choice is one counter increment and one mask test, so the change adds almost nothing to the put path. Get and put both came in slightly faster, most visibly at put p99. The likely reason is that a new key at the probationary head becomes the next victim, so during miss runs put keeps reusing the same arena slot and its neighbors, which are already hot in the CPU cache. The gain is small enough that some of it may be noise.

## Hurt

- hit-rate: With head insertion, a new key stays resident only until the next miss. It can only reach the protected segment if it is hit before any other key misses. In the parent, a new key had the whole probationary segment (about 20% of capacity, since protected is capped at 80%) as its window for a second touch. Keys on LIRS workload-a whose reuse distance is short but longer than one miss could earn that touch in the parent, and here they are evicted first. The parent's probation-then-protect filter already kept one-touch scan keys away from reused keys, so the bimodal layer added little scan protection. Its main effect was to shrink the admission window for keys that were worth keeping.
- metadata-size: `Node<u64,u64>` is unchanged, so the summary dimension did not move. The `LruCache` struct grew, though: the new `insert_tick: u32` comes after the four `u32` head/tail fields, and the following `usize` alignment adds padding, so the struct grows by a full word rather than the 4 bytes the field needs.

## Takeaway

Putting BIP-style LRU-end insertion inside SLRU's probationary segment did not stack with SLRU. The two mechanisms overlap on scan protection. BIP also takes away the probation window that SLRU uses to find second touches, so the net hit-rate effect on this trace is negative. If later variants want to keep a long loop from flushing probation, they should do it without making new keys the immediate victim, for example by resizing probation or by filtering admission. A 1-in-32 MRU trickle is too slow to make up for a window of one miss.
