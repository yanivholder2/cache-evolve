# Variant v003 Insight

## Helped

_None measured._ The two dimensions H03 was meant to move, `per-op-time` and `metadata-size`, are missing from v003's fitness vector, so there is no evidence either way. The code does make the changes the hypothesis called for: the Fx-style multiply-rotate hasher replaces SipHash, `u32` links with a `NIL` sentinel replace `Option<usize>`, `get` and the existing-key `put` skip the splice when the node is already the tail, and eviction uses `mem::replace` to swap the key.

## Hurt

_None._ `hit-rate` did not move, as intended. The change only alters how entries are stored, hashed and linked. Eviction is still strict LRU from `head`, and every promotion still ends with the node at `tail`; the tail fast path only skips moves that would change nothing. So the order of hits and misses on the trace is identical to the parent's.

## Takeaway

v003 shows that this group of representation changes (Fx hashing, `u32` sentinel links, the MRU fast path and the eviction key swap) leaves LRU behaviour exactly as it was, so it is safe to build on for correctness. It does not yet show whether these changes make the cache faster or smaller, which was the whole point of this control arm. `per-op-time` and `metadata-size` need to be re-measured before the claim that get p50 drops below the baseline can be accepted or rejected.
