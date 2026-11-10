# Cycle 0008 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002. v001 raised `MAX_FREQ` from 7 to 15, v002 gave graduating keys `freq = 1` instead of 0, and v003 keyed the ghost structures by 32-bit fingerprints. None beat the parent. v001 and v002 lost summary hit rate, including on their target cell, hm_0 at 16384. v003 kept hit rate identical but made `put` slower. All three are Winners by rank only.

## Patterns

- Both levers for main tenure are now used up from both directions. On freq aging, halving (cycle 7) hurt and a cap of 15 hurt, so 7 with decrement-by-one is the local optimum. On graduation credit, full carry-over (cycle 3) hurt and a single credit hurt, so resetting to 0 is right. More tenure that keys have not earned in main, whether old hot keys keep it or newly graduated keys get it, blocks keys that are actually reused in a small, fast-turning main. Cycle 7's call for "more tenure" was only half right. What matters is tenure earned in main, not more of it.
- 131072 stayed flat again under every policy change. Main retention at the large size is saturated, and all of the remaining hit-rate room is at 16384.
- `put` p99 rose in all three variants. In v001 and v002 the cause was longer sweeps: more decrement-and-reinsert passes before a victim is found. In v003 it was extra hashing. The tail tracks work per miss, which is hash computations and sweep steps, not bytes touched. Making the ghost footprint smaller saves nothing when the keys are already 8 bytes and cheap to copy.
- Per-entry metadata stayed flat. Every diff fit in the existing 24-byte entry.

## Open questions

- Would a fingerprint ghost table with an identity/pass-through hasher, so the fingerprint is the hash and there is one hash per miss, finally beat the parent on `put`?
- Is there room left at hm_0 16384 for a lever that is neither tenure nor credit, for example a small/main split that depends on cache size, or should this policy family be treated as converged on hit rate?

## Continuation

Keep cycle-5 v002 as the parent. Drop all freq-cap, aging-rate, and graduation-credit variants, since each has now failed in both directions. On hit rate, the only untested lever is a small/main split that depends on cache size, aimed at 16384. If that fails, treat hit rate as converged. On timing, retry fingerprints only with a pass-through hasher, or target removing a hash or sweep step per miss. Keep eviction work bounded, and give no timing credit to policy-only diffs.
