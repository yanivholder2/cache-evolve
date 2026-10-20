# Variant v002 Insight

Note: I could not list or read the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- hit-rate: Unchanged. The hypothesis kept sketch semantics, the admission rule and the counter widths fixed, and the fitness vector agrees.
- metadata-size: Unchanged. The hypothesis forbade changes to `Node` size, the sketch and the cache struct, and none occurred.

## Hurt

- per-op-time (get): The `get` fast-path restructure did not make `get` faster. Median and tail latency both rose against the parent, so the `#[inline(always)]`, cold-miss and hash-reuse changes did not remove the real cost. They may have bloated the hot path or disturbed code layout.
- per-op-time (put): `put` also slowed, with a worse tail. `put` shares the inlined sketch helpers and the lookup path with `get`, so forcing `#[inline(always)]` on `index`, `increment` and `estimate` plausibly grew its code too. Another possibility is measurement noise, since a slowdown this broad suggests a shared cause.

## Takeaway

Forcing inlining and branch hints onto `get` and the shared sketch helpers did not speed up this cache and may have slowed it. The parent's `get` was not bottlenecked on call overhead, duplicate hashing or branch layout. Future latency work should start from measurement of where time goes, and `#[inline(always)]` on helpers shared by `get` and `put` should be treated as a risk.
