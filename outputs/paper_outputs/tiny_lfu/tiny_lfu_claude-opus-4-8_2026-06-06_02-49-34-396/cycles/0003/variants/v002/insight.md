# Variant v002 Insight

## Helped

- metadata-size: Replacing each `Node`'s two `Option<usize>` links (16 B apiece, no niche) with `u32` sentinel indices halved the node from 48 to 24 bytes, and shrinking `head`/`tail` to `u32` also trimmed the cache struct.
- per-op-time: The `== NIL` branch on a 4-byte index replaced `match Option`/`expect`, slightly tightening the hot `unlink`/`push_back` path on both get and put.

## Hurt

- (none): No dimension regressed.

## Takeaway

The metadata-size axis was pinned at 48 only because no prior variant touched node layout; encoding linked-list links as `u32` sentinels instead of `Option<usize>` directly halves per-entry overhead while preserving exact LRU ordering and admission, so hit-rate is untouched. This confirms the link representation, not the sketch, was the cheap win on the size axis.
