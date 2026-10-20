## Recurring wins

- One-shot-insert filtering is the most consistent positive direction. Probationary-first eviction (Cycle 0003), the shared-frequency admission gate (Cycles 0004 and 0005), and eviction-next placement of misses (Cycle 0005) each produced a hit-rate gain on workload-a. All of them keep one-shot inserts from displacing entries with proven reuse.
- Scan resistance beats ranking changes. The gains came from changing which entries get displaced, not from changing how existing rankings are computed.
- Admission decisions work when both frequency estimates sit on one scale. The shared counter (Cycle 0004) succeeded where the earlier mismatched-scale gate (Cycle 0003) lost.

## Recurring failures

- Hit-rate-only geometry tweaks have not paid off. Bucket repartitioning (Cycle 0001) and fingerprint filtering (Cycle 0002) gave marginal or no movement on the one dimension measured.
- Aging-based frequency policies have produced no measurable hit-rate gain in every cycle that tried them (Cycles 0001, 0002). Whether the evaluation selects them at all is unresolved.
- Bounding the victim scan does not change hit-rate (Cycles 0003, 0004). Exact-minimum eviction appears to buy little, but its cost savings have never been measured.
- The protected tier with second-hit promotion has failed repeatedly (Cycles 0004, 0005, 0006). Removing demote-on-promotion and shrinking the protected region both left it flat or below parent, so the second-hit signal or demotion rule is the likely cause.
- Bounded or compressed frequency metadata lowered hit-rate (Cycle 0006). Aliasing, saturation, and halving were changed together, so no single cause is isolated.
- Front-of-probation insertion lowered hit-rate (Cycle 0006), contrary to the eviction-next placement that gained in Cycle 0005.

## Open questions

- Winner selection does not match the hit-rate evidence (Cycles 0004, 0006). Confirm the selection rule before treating any Winner as a precedent. Cycle 0005's Winner also has no recorded magnitude.
- Per-op time and metadata size have never been measured in any cycle (Cycles 0001 through 0006). Every cost-motivated hypothesis is therefore untested.
- Do the admission gate and eviction-next placement stack or overlap? Testing each alone against the parent would settle it.
- Do the one-shot-insert gains hold beyond workload-a?
- Are the overlays for some variants active under the harness? Several insights could not confirm this, so flat results may be non-applications rather than nulls.

## Per-policy notes

- Aging / AgedLfu: two cycles of null results on hit-rate. Halving frequency and whether it fires under the harness are unverified. Do not retry without logging activity.
- Bucket / fixed-width geometry: repartitioning gave only a marginal gain (Cycle 0001). Further geometry changes need a paired cost measurement.
- Probationary / two-region: the strongest line of gains. Probationary-first eviction was the base for the later admission and insert-position wins.
- Protected tier (second-hit promotion, demote-on-promotion): repeatedly flat or negative. Changing the cap alone did not help. Retry only with a changed second-hit or demotion rule.
- Frequency admission gate: positive once both estimates share a scale (Cycle 0004). Bounded or direct-mapped counters have hurt (Cycle 0006); keep the counter exact until cost is measured.
- Insert position: eviction-next placement helped (Cycle 0005); front-of-probation placement hurt (Cycle 0006). Always state the insert position explicitly.
