# Variant v002 Insight

## Helped

- metadata-size: The rotation state was placed on the bucket (`victim_cursor: Cell<u8>`) rather than on each `Item`, so per-item metadata is untouched and `Item<u64,u64>` stays at the parent's footprint.

## Hurt

- hit-rate: CLOCK-style rotation among untouched slots discards the implicit age ordering that made v003's leftmost-first work — because `get_from_position` moves hit items to the back, the front of the bucket naturally accumulates the most-senior untouched slots, and "leftmost untouched" already encoded "longest-resident untouched"; rotating gives those senior never-hit slots a free reprieve and evicts younger untouched slots that hadn't yet had their chance to earn a hit.

## Takeaway

v003's leftmost-bias wasn't an arbitrary tie-break — under ClassicLRU's move-to-back dynamics it was a free age signal, with the oldest never-hit slot landing at index 0. Replacing it with stateless rotation swaps "evict the most-senior one-shot first" for "evict whichever one-shot the cursor points at," which is strictly less informed. For Dash bucket-movement, when an existing index ordering is correlated with a useful signal, fairness-style spreading destroys information rather than refining it; CLOCK helps only when slot position carries none.
