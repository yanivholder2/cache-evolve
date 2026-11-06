# Variant v003 Insight

## Helped

- per-op-time (get p99): The get tail fell back down. The get path (`map.get` → `on_hit`) is the same code as the parent's, so this most likely shows that the parent's p99 jump was noise or a layout artifact, as the v002 insight suspected. It is probably not a gain from the halved bitset. The header did change (`capacity: u32`, new `ghost_cursor`), so a layout effect can't be fully ruled out.
- metadata-size: The ghost bytes halved at both sizes (16 KB → 8 KB at 16K, 64 KB → 32 KB at 131K). `Node<u64,u64>` stays at 24 bytes and the header stays within 128 bytes, because narrowing `capacity` to `u32` made room for `ghost_cursor`. The headline dimension only counts the node, so the measured summary doesn't move.

## Hurt

- per-op-time (get p50): Get p50 rose clearly, even though the ghost is now half the parent's size and get never touches it. This contradicts the v002 explanation that a smaller ghost frees cache lines for the nodes and map buckets and so speeds up get. Either that link is weak or the parent's p50 drop was partly noise. The fact that the change goes the wrong way is the useful signal here.
- per-op-time (put p50): Put p50 rose. This fits the extra work in the `Vacant` eviction arm. Every evicting put now reads and writes the cursor and takes a branch, every 16th put also zeroes a word, and the lookup does a shift-mask read-modify-write in place of a byte compare. A table half the size did not offset that added work.
- hit-rate: The summary fell slightly but stayed within the hypothesis's ±0.0005 budget. The biggest single drop is at hm_0@16K, which by itself goes just past the budget. Two things cause this. A 1-bit ghost with no tag grants a reprieve to any key that hashes onto a set bit. And the rotating word clear forgets real ghosts early, whenever a key's word is swept before the key comes back. web_0@131K improved slightly. Unlike the bit-identical v002 result, these moves show that the hit-rate harness does run the ghost path.

## Takeaway

A bitset aged by a rotating cursor can replace an 8-bit tag table at half the bytes with almost no hit-rate cost. The extra 4× bits keep false positives low enough that one tagless bit per remembered key works. But making the ghost smaller did not make get faster this time, and the aging bookkeeping made put slower. So "shrink side structures to speed up get" is not confirmed, and the footprint lever looks used up in this range. More ghost shrinking should be judged on its hit-rate and put cost, not on any expected get-latency gain. Before relying on a cause, re-measure get p50 for v002 and v003 side by side, since this cycle's change goes the opposite way from last cycle's.
