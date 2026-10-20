# Variant v001 Insight

## Helped

- per-op-time (`get`): The hit path now writes only to the 256-byte `visited` bitset, which stays in L1, and no longer writes `links[idx]`. The test-before-set also stops repeated hits on hot keys from dirtying the word. This removes most of the second cold cache line that v002's insight blamed for its `get` regression, and both p50 and p99 improved. The gain is partial rather than a full recovery: a hit still pays a map probe, a `nodes[idx]` line, and a bounds-checked read-modify-write on a separate array.
- per-op-time (`put` p99): The tail improved, probably because the SIEVE hand in `find_victim` now clears VISITED bits in the dense bitset. It reads `links` but never writes to it, so a long sweep over many visited nodes writes to a few bitset words instead of dirtying one `links` line per node.
- hit-rate: Held exactly, which was the intended result. VISITED keeps the same meaning and lifecycle: it is cleared on slot reuse and push, and tested then cleared in the same order on small-head pops and in the hand walk. Only its storage moved, so eviction order matches v002.

## Hurt

- per-op-time (`put` p50): The common miss path now does an extra `vbit_clear` on a third array in addition to the `links` write, and small-head promotion touches both `links` and the bitset. This adds a little fixed work to every insert, and the median `put` got slightly slower even though the tail improved.
- metadata-size (struct): `LruCache<u64,u64>` grew again because the `visited` `Vec` adds another pointer/capacity/length header. Per-node bytes stay at 16, and the bitset's heap cost is only 1 bit per slot. The per-instance fixed cost keeps rising with each side array, though.

## Takeaway

Moving VISITED from the per-slot `u32` link word into a dense L1-resident bitset makes the `get` hit path skip `links` entirely, and keeps the 16-byte node and the exact S3-FIFO eviction order. This confirms that much of v002's `get` penalty came from writing to the `links` array. It does not fully remove that penalty, because a hit still touches the value line and a second structure. The cost moves to the miss path: each insert now does one more small write, so `put` p50 rose a little while `put` p99 benefited from the bitset-only hand sweep. A bitset is the right home for a hot, write-on-hit flag when per-node size is fixed. Further `get` gains would need to avoid the separate structure altogether, for example by keeping the flag on the value's cache line.
