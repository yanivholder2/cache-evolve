# Variant v003 Insight

## Helped

(No measured dimension improved. The eval covers hit-rate only. The dimensions this hypothesis aimed at, per-op-time and metadata-size, are not in the fitness vector, so the claimed cost wins are still unmeasured.

Two cost changes are visible in the code:
- `put()` and `ghost_insert()` now each hash the key once with a multiply-rotate `FpHasher` plus a splitmix64 finalizer, instead of SipHash.
- `ghost_map` lookups now go through `IdentityU32Hasher`, a single odd-constant multiply, instead of a second SipHash over the fingerprint.

On the metadata side, `ghost_ring` drops from `u64` to `u32` entries, and the `ghost_map` key also shrinks from `u64` to `u32`.)

## Hurt

(No measured dimension regressed. Hit-rate did not move from the parent, which matches the hypothesis's "policy-neutral" claim. The change only touched how ghost fingerprints are computed, stored and looked up. `evict_main` clear-on-pass SIEVE, graduation at `freq > 1`, head/freq-0 re-admission, ghosting of main victims, the 5% split and `MAX_FREQ = 7` are all the same as before. The lockstep `RefS3Fifo` test still checks small/main order, per-entry freq, hand position and the exact ghost-ring layout against the new `u32` fingerprints. The odds of a 32-bit collision across about 1946 live ghost entries are too small to move the outcome on workload-a.)

## Takeaway

Narrowing ghost fingerprints to 32 bits and replacing SipHash with an FxHash-style mixer behind an identity-hashed map leaves the clear-on-pass SIEVE policy unchanged on the cycle-8 lineage. That confirms on v001 the neutrality cycle 8 had only checked on cycle-7 code. So v003 can stand in for cycle/0008/v001 as a cheaper base for later policy variants. But its per-op-time and metadata-size benefits stay theoretical until the eval measures those dimensions.
