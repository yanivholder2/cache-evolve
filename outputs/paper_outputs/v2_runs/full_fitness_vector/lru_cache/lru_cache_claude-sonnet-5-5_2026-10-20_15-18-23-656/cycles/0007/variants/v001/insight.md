# Variant v001 Insight

## Helped

(none — no dimension improved over the parent)

## Hurt

- hit-rate: Sending ghost-hit keys straight to the protected segment pushed out protected LRU entries to make room. On this trace, evicted probationary keys that return are mostly not hot enough to deserve protection, so the demotions cost more than the extra hits gained.
- metadata-size: The new `ghost: Vec<u32>` and `ghost_mask` fields enlarged the `LruCache` struct. `Node` is unchanged, so the per-node summary didn't move. The heap table is sized at the next power of two of capacity, which is extra memory the struct-size metric doesn't count.
- per-op-time: Every put-miss now computes `hash_one` for the new key. Every probationary eviction computes it again for the victim, which adds a random-access ghost array read and write on top of the existing map lookup. Both get and put latencies rose, probably because the larger struct and the extra table access on the put path add cache pressure.

## Takeaway

A direct-mapped ghost fingerprint table with immediate protected admission was a net loss here. The hash and table cost on every miss outweighed any gain from recognising re-referenced evictees, and the protected admissions lowered hit-rate rather than raising it. A ghost history would need a gentler promotion rule, or a cheaper way to detect re-references, to pay off.
