# Variant v002 Insight

## Helped

- metadata-size: Narrowing the arena links, list heads and tails, and map values from `usize` to `u32` with a `NIL = u32::MAX` sentinel shrinks each `Node` by padding it down to the expected layout, and the same narrowing shrinks the `LruCache` struct itself.
- per-op-time: The `put` median and tail both moved down slightly, and the `get` median held flat, so the width conversions the hypothesis flagged as a risk did not land on the hot path. The `get` tail moved up by a few nanoseconds, which is small enough that it may be noise.

## Hurt

## Takeaway

The narrowing is policy-neutral, as expected, since the eviction logic in `select_victim`, `demote_prot_head`, `unlink`, and `push_back` is untouched apart from types. The main new behavior is the `capacity > u32::MAX` rejection in `try_new`, which turns very large capacities into a `None` and is the one place this change can fail where v001 could not. The inline `as usize` and `as u32` casts cost essentially nothing measurable here, which suggests the earlier slowdowns in cycle/0002/v003 and cycle/0004/v002 came from the accessor structure rather than from the casts themselves.
