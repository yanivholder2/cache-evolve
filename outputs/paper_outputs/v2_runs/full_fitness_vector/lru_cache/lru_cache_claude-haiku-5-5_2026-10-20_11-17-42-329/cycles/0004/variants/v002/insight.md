# Variant v002 Insight

## Helped

## Hurt

- per-op-time: `get` got slower relative to the parent, even though the hypothesis's change was meant to cut repeated arena index conversions in `get`, `unlink`, and `push_back` down to one `node()` lookup per operation.
- per-op-time: `put` also got slower, although the change was scoped to the move-to-front and link-splicing paths. This points at the new `node()`/`node_mut()` helpers or the reference-reuse pattern affecting code generation on both hot paths, not at a reduction in arena accesses.

## Takeaway

Hoisting the index conversions into two `#[inline(always)]` accessors did not reduce `get` cost on top of the narrowed u32 links, so the access-count explanation from cycle 0002 does not hold on its own. Hit-rate and metadata-size stayed identical to the parent, as expected from a change that touched no policy and added no fields. The next experiment should test layout or codegen effects directly (for example, comparing the parent's inline index casts against the helper form in isolation) before stacking more access-pattern rewrites.
