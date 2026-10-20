# Variant v003 Insight

## Helped

- metadata-size: Removing `prev` and `next` from `Node` leaves only `key` and `value` in the arena element, so the struct shrinks from the parent's packed 24-byte layout. The per-slot reference state moved into a parallel `refs: Vec<u8>`, which the Node-size metric does not count, so the real per-slot footprint saving is smaller than the struct number suggests but still positive.

## Hurt

- hit-rate: Replacing the parent's two-segment SLRU (probationary and protected lists with a single second chance) with a single CLOCK ring of 2-bit counters removes the protected segment that kept rescued keys safe from being evicted by a stream of one-hit keys. A key's counter is only decremented one step per sweep pass, so recently hot keys compete on equal footing with newer entries, which is the likely source of the drop on workload-a.
- per-op-time: `get` got slower because every hit now does a saturating increment into a separate `refs` vector, touching a second allocation that the parent's single OR on the node's `prev` word avoided. `put` tail latency worsened because a miss at capacity can now sweep several slots, decrementing counters as it goes, where the parent did an O(1) head check.

## Takeaway

The CLOCK rewrite buys a smaller node struct by trading away the segmented protection that the parent relied on for its hit rate, so the hypothesis's "small hit-rate cost" was an underestimate for this workload. The metadata win is also partly illusory once the `refs` allocation is counted, which means the hypothesis's claim that the trade is worth it needs the full per-slot footprint, not the struct size, to judge.
