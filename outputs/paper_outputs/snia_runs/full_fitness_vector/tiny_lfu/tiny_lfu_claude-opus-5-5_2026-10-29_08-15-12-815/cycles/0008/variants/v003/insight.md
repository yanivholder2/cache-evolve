# Variant v003 Insight

## Helped

(none — metadata-size did not move: `REF_BIT` lives in the spare top bit of `Node::next`, so the node stays at 24 bytes and the sketch and cache header are unchanged)

## Hurt

- per-op-time (get p50, summary): The protected-hit fast path (`next |= REF_BIT`) was meant to remove neighbour writes. But the masking needed to carry the bit turned the parent's blind link stores into read-modify-writes. In `unlink`, `p.next = (p.next & REF_BIT) | next` now has to load the prev neighbour first. In `push_back`, `t.next = (t.next & REF_BIT) | idx` now has to load the old tail first. The parent's protected-hit stores to the prev neighbour and the old tail did not wait on those cache lines. The store buffer absorbed them, so they were not the stalls the hypothesis blamed. Every window, probation and eviction relink now waits on those loads. That more than cancels what the protected-hit shortcut saved.
- per-op-time (get p99): Promotions from probation now run `demote_protected_head`. Under a hot working set most protected heads are referenced, so the loop often goes the full `MAX_SECOND_CHANCE = 4` rotations. That means up to five unlink/push_back pairs, each with the new RMW loads, where the parent did one. The bound fixed the worst case, but it fixed it above the parent's cost. The tail spike moved into promotion instead of going away.
- per-op-time (put p50/p99): Put has no protected-only fast path to benefit from. Its miss path (`evict_one` → `unlink` ×2 → `push_back` ×2) pays the new masked RMW on every relink. Put-on-existing-key also goes through `on_hit` and so gets the second-chance loop on promotions. That is why the put tail widened the most of any latency number.
- hit-rate (all four cells, summary): Every cell slipped slightly, as the hypothesis warned. There are two reasons. Protected order now reflects when a node was promoted or last rotated, not when it was last hit. And after four referenced heads the bounded loop demotes the next head even if its bit is set, so a recently hit node can be sent to probation. Both effects make protected eviction coarser than true LRU, at both cache sizes and on both traces.

## Takeaway

On this 32-bit-link layout, a CLOCK reference bit kept in `next` costs more than it saves. The parent's move-to-tail on protected hits was mostly blind stores that did not stall. Carrying the bit forces a load-before-store into every relink on every path, and the second-chance loop adds more relinks on promotion. Removing the protected-hit pointer chase only pays off if the reference bit is stored somewhere the link writes never have to preserve. A separate bitset or byte array indexed by slot would do that, as long as the parent's plain `next` stores stay in place.
