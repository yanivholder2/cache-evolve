# Variant v001 Insight

## Helped

- **per-op-time**: Replacing v002's `u32`/`NULL_IDX` sentinel comparisons with `Option<NonZeroU32>` and idiomatic `match` arms restored the compiler's well-predicted discriminant-test codegen, cutting p50 latency for both `get` and `put` back toward the pre-v002 baseline.

## Hurt

_(no dimension hurt)_

## Takeaway

`Option<NonZeroU32>` with 1-based index encoding achieves the same 4-byte field width as v002's raw `u32` sentinel — confirmed by the unchanged 24-byte `Node` size — while recovering the hot-path speed lost in v002. This isolates the root cause of v002's latency regression: it was the explicit `== NULL_IDX` guard branches, not the smaller node footprint, that degraded per-op-time. The niche optimization makes `Option<NonZeroU32>` a strictly better encoding than a raw sentinel integer when idiomatic `match` codegen matters.
