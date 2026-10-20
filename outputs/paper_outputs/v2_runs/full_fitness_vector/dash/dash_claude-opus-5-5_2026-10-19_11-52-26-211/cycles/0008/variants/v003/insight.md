# Variant v003 Insight

## Helped

- hit-rate (held, as required): hit rate matches the parent exactly. This shows the change from `Vec<Box<dyn Bucket>>` to `Vec<DashBucket>` did not change behaviour. The new inherent methods on `DashBucket` (`get_position`, `get_from_position`, `put_at`, …) copy the trait semantics, and `put_at` / `evict_item` delegate straight to the trait defaults. Recency touches and victim choice are unchanged.
- metadata-size (held): `Item` and `Dash<u64,u64,DashSegment>` are the same size as before. The layout change sits behind `SegmentCore`'s `Vec`s, as the hypothesis predicted.

## Hurt

- per-op-time (get p50): the hypothesis is **refuted**. `get` p50 did not drop below the parent's level, and it is effectively flat. Removing the `Box` took away only one of the two pointer chases per probe. `DashBucket` still keeps its slots in a separate heap `Vec<Item>`, so each probe still loads a pointer from the bucket struct and then follows it to the items buffer. On a hit, `get_from_position` still makes a vtable call through the boxed `dyn EvictionPolicy`. So the indirections that cost the most on a 2-slot scan are still there.
- per-op-time (get p99): the `get` tail got worse. The contiguous array now holds 48-byte bucket headers (Vec + `max_size` + fat `Box<dyn EvictionPolicy>` pointer) instead of 8-byte `Box` pointers. A 1–4 bucket probe sequence (target → probing → `s0` → `s1`) can now cross more cache lines of headers, and it still pays the item-buffer miss each time. Whether this costs more than it saves can't be confirmed from one run.
- per-op-time (put p50/p99): `put` got worse at both p50 and p99, even though the write logic is unchanged (`put_at` still runs the trait-default body). This is most likely a codegen or layout effect, such as more inlining in the put/demotion path or the larger bucket stride, not new work. The parent's unusually low `put` p99 was already marked as possible variance, so part of this tail change may just be the baseline returning to normal. Re-measure before blaming the change.

## Takeaway

Making the bucket layer concrete does not help when the bucket is itself only a header around a heap `Vec<Item>` and a boxed `dyn EvictionPolicy`. The pointer chase and the dynamic call just move one level down. The `Box<dyn Bucket>` hop was not on the critical path: either the compiler already devirtualised it, or the item-buffer miss dominates. To make probes cheaper, the slots themselves have to be stored inline, for example as a fixed `[Item; N]` array or a flat per-segment item array indexed by bucket. The policy also needs to be static, not boxed per bucket. Swapping `Box<dyn Bucket>` for the same-shaped concrete struct is not enough.
