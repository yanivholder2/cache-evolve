# Variant v002 Insight

## Helped

- hit-rate: Replacing the parent's unconditional re-admission into protected with a gated scheme — single ghost hits go to the probationary MRU, and only repeated ghost evidence (`GHOST_PROMOTE_HITS = 2`) earns a shielded slot at the *protected LRU end* — stops transient post-eviction re-references from displacing genuinely hot protected keys, lifting hit-rate from below the SLRU baseline back over the 0.724 record.
- per-op-time: Routing single ghost hits to probation instead of `push_back` into protected means the common new-key path no longer bumps `prot_len` or fires `demote_protected_lru`, and the protected list stays shorter and more stable, so both `get` and `put` do less link surgery (p50 and p99 fell on both ops).

## Hurt

- (none) — no dimension regressed. The extra per-ghost `u8` counter and the switch from removing keys in `ghost_admit` to bumping and retaining them added bookkeeping, but it did not surface as a latency or size cost.

## Takeaway

Gating ghost re-admission on repeated evidence — and demoting the single-hit case to plain probation — is what turns the reuse-distance signal from the parent's hit-rate regression into a small gain: keeping transient re-references out of protected both recovers hit-rate and, by avoiding protected-segment churn and demotions, actually speeds up the per-op path rather than taxing it. metadata-size did not move because ghost state lives outside `Node` in the separate `ghost_queue`/`ghost_set` structures.
