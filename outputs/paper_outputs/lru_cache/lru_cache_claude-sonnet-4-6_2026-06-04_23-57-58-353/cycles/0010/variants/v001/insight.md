# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Replacing the inert referenced-bit with IRR timestamps as the tie-breaker in both `demote_prot_head()` and the probationary eviction scan produced a small regression, likely because ghost-admitted entries start with `prev_access = 0` (IRR = `u64::MAX`), making them appear maximally cold at any frequency and eligible for immediate demotion when they tie with tenured protected entries — the opposite of what ghost admission intends.

## Takeaway

IRR timestamps do not improve the tie-breaker in this ghost-admission context because the most plentiful source of `prev_access = 0` entries is ghost-admitted keys: they re-enter protected fresh, with no second timestamp, and the `u64::MAX` sentinel the code uses to encode "never accessed twice" is indistinguishable from a genuinely cold newcomer. The ghost mechanism is designed to accelerate warm re-arrivals into protection, but the IRR scoring then undermines it by treating those same entries as the coldest candidates during a same-frequency demotion scan. Same-frequency ties inside the narrow LRU-quarter window remain rare on workload-a regardless of tie-breaker choice, so neither the IRR change nor the referenced-bit before it can move the needle — the collision pattern that would activate a secondary signal is not present at this cache size and trace.
