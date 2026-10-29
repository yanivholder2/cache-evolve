# Variant v002 Insight

## Helped

- metadata-size: The node now has two `u32` links and no `Option`. `next` drops its `Option<usize>` discriminant and uses the same `NIL` sentinel as `prev`, and both links shrink to 32 bits. The region tag moves to bits 30–31 of `prev`, so `Node<u64,u64>` packs to key + value + two 4-byte links with no padding. The 2^30 index limit this creates is enforced in `try_new`, so the space saving does not risk a sentinel collision for any realistic capacity.

## Hurt

- per-op-time (get and put): Latency rose a lot on both ops, at both p50 and p99. The hypothesis predicted this would not happen, and the diff does not explain it. The hot paths (`get`, `on_hit`, `unlink`, `push_back`, `evict_one`) match v001 instruction for instruction except for `u32`↔`usize` casts and 32-bit masks. On x86-64 those casts are free zero-extensions. A smaller node should also mean fewer cache misses on the 131072-entry arena, not more. There is no stated mechanism in the change for a regression this large and this uniform. The likeliest causes are a benchmark-environment effect (noise, CPU frequency or co-tenancy, a different build profile) or a codegen change outside the edited logic. This needs re-measuring against v001 in the same run before it counts as a real cost of 32-bit links.

## Takeaway

Narrowing the arena links to sentinel-encoded `u32`s is a clean, behaviour-preserving space win. Hit-rates are bit-identical to v001, which confirms the `NIL`/region-tag encoding on both links is correct. The latency claim did not hold up, but the code gives no plausible reason for it to fail. So the next step is to benchmark v001 and v002 back to back in the same session (and check the generated assembly for `unlink`/`push_back`) before treating "compact links are slower" as a lesson. If the regression reproduces, check codegen first (for example, bounds checks the compiler no longer removes after the `as usize` widening), not memory layout.
