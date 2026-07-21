# Variant v002 Insight

## Helped

*(No dimension clearly improved.)*

## Hurt

- **metadata-size**: Adding `ref_bits: Vec<u8>` to `LruCache` inflated the struct from 120 to 144 bytes — a `Vec` header costs 24 bytes on 64-bit regardless of how few bytes the bitmap itself holds, and the policy change produced no gain to justify that overhead.
- **per-op-time**: The get p50 rose from below timer resolution to 100 ns; replacing `unlink`+`push_back_prot` with a single bit-or on protected hits was supposed to be cheaper, but the CLOCK sweep loop (up to 8 iterations of conditional test, clear, unlink, push_back_prot) during each demotion event, plus the extra bitmap index calculation on every protected access, added enough branch and memory overhead to raise the measured floor.
- **hit-rate**: Marginally lower (within noise), meaning the second-chance CLOCK semantics provided no benefit on workload-a's access distribution.

## Takeaway

Replacing eager pointer relinking on protected hits with a lazy bitmap bit-flip plus a bounded CLOCK sweep at demotion time did not reduce per-operation cost: the 24-byte `Vec` header for `ref_bits` bloated the struct, and the worst-case eight-iteration sweep at demotion time introduced more work than the per-hit `unlink`+`push_back_prot` it was meant to amortize away. The second-chance eviction policy also failed to improve hit-rate on workload-a, suggesting the trace's hot set is stable enough that simple LRU demotion already makes correct choices and second chances offer no additional protection.
