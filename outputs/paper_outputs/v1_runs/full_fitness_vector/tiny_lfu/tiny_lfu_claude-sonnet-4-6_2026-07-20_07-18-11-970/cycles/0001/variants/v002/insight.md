# Variant v002 Insight

## Helped

- **metadata-size**: Replacing `Option<usize>` prev/next fields (16 bytes each on 64-bit, no niche) with `u32` sentinel fields (4 bytes each) cut `Node<u64,u64>` exactly in half, and the same transformation on `TinyLfuCache.head`/`tail` also shrank the struct by 24 bytes.

## Hurt

- **per-op-time**: Every `unlink` and `push_back` now emits explicit `as usize` / `as u32` casts and compares against the `NULL_IDX` sentinel, replacing idiomatic Rust `Option` pattern-matching that the compiler previously reduced to well-predicted branches; the result is a roughly 2× slowdown at both `get` and `put`.

## Takeaway

Swapping `Option<usize>` links for `u32` sentinels achieves the predicted 50 % reduction in per-node metadata but doubles operation latency, suggesting that the compiler's `Option`-based codegen (tight discriminant tests, strong branch prediction) outweighs the cache-density benefit of smaller nodes at this working-set size. Future work should either accept the size–speed trade-off or look for a representation (e.g. `NonZero<u32>` to restore niche optimizations) that keeps the node compact without regressing hot-path instruction quality.
