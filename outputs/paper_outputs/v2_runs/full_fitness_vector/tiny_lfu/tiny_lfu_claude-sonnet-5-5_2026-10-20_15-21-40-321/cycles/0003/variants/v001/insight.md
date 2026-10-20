# Variant v001 Insight

Note: I could not list or diff the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- metadata-size: Replacing the two `Option<usize>` prev/next links in `Node` with `u32` indices (sentinel `NIL = u32::MAX`) shrank `Node<u64,u64>`. The 16-byte option-plus-pointer fields became 4 bytes each. The `TinyLfuCache` struct also shrank, because `head` and `tail` got the same treatment. The sketch heap is unchanged, since the sketch was not touched.
- hit-rate: Nothing moved, which is the intended result. LRU order, the sketch and the `incoming_freq > victim_freq` admission rule were left alone, so behavior is identical to the parent.

## Hurt

- per-op-time: Both `get` and `put` got slower, and the tails got worse too. The hypothesis expected fewer cache lines per list operation. The cache is only 2048 entries, so the node array likely fits in cache already and there was little to save. The likely cost is the new work at each link site. Every `u32` link needs a `NIL` compare and an `as usize` widening where `Option` matching used to be. `put` is the op that relinks most nodes, so it may have slowed most.

## Takeaway

Narrowing the list links to `u32` indices did what it was meant to on metadata-size. It brought `Node` well below the 48 bytes the metric had been stuck at, with no hit-rate change. It also gave back per-op latency that the previous cycle had won. The locality benefit does not appear at this working-set size, so the layout change trades time for space. Compare the size gain against the latency loss before treating this as a pure win.
