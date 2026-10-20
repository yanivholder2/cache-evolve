# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:35:28+00:00

## timeline

### 14:35:35 thinking



### 14:35:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- One-shot-insert filtering is the most consistent positive direction. Probationary-first eviction (Cycle 0003), the shared-frequency admission gate (Cycles 0004 and 0005), and eviction-next placement of misses (Cycle 0005) each produced a hit-rate gain on workload-a. All of them keep one-shot inserts from displacing entries with proven reuse.\n- Scan resistance beats ranking changes. The gains came from changing which entries get displaced, not from changing how existing rankings are computed.\n- Admission decisions work when both frequency estimates sit on one scale. The shared counter (Cycle 0004) succeeded where the earlier mismatched-scale gate (Cycle 0003) lost.\n\n## Recurring failures\n\n- Hit-rate-only geometry tweaks have not paid off. Bucket repartitioning (Cycle 0001) and fingerprint filtering (Cycle 0002) gave marginal or no movement on the one dimension measured.\n- Aging-based frequency policies have produced no measurable hit-rate gain in every cycle that tried them (Cycles 0001, 0002). Whether the evaluation selects them at all is unresolved.\n- Bounding the victim scan does not change hit-rate (Cycles 0003, 0004). Exact-minimum eviction appears to buy little, but its cost savings have never been measured.\n- The protected tier with second-hit promotion has failed repeatedly (Cycles 0004, 0005, 0006). Removing demote-on-promotion and shrinking the protected region both left it flat or below parent, so the second-hit signal or demotion rule is the likely cause.\n- Bounded or compressed frequency metadata lowered hit-rate (Cycle 0006). Aliasing, saturation, and halving were changed together, so no single cause is isolated.\n- Front-of-probation insertion lowered hit-rate (Cycle 0006), contrary to the eviction-next placement that gained in Cycle 0005.\n\n## Open questions\n\n- Winner selection does not match the hit-rate evidence (Cycles 0004, 0006). Confirm the selection rule before treating any Winner as a precedent. Cycle 0005's Winner also has no recorded magnitude.\n- Per-op time and metadata size have never been measured in any cycle (Cycles 0001 through 0006). Every cost-motivated hypothesis is therefore untested.\n- Do the admission gate and eviction-next placement stack or overlap? Testing each alone against the parent would settle it.\n- Do the one-shot-insert gains hold beyond workload-a?\n- Are the overlays for some variants active under the harness? Several insights could not confirm this, so flat results may be non-applications rather than nulls.\n\n## Per-policy notes\n\n- Aging / AgedLfu: two cycles of null results on hit-rate. Halving frequency and whether it fires under the harness are unverified. Do not retry without logging activity.\n- Bucket / fixed-width geometry: repartitioning gave only a marginal gain (Cycle 0001). Further geometry changes need a paired cost measurement.\n- Probationary / two-region: the strongest line of gains. Probationary-first eviction was the base for the later admission and insert-position wins.\n- Protected tier (second-hit promotion, demote-on-promotion): repeatedly flat or negative. Changing the cap alone did not help. Retry only with a changed second-hit or demotion rule.\n- Frequency admission gate: positive once both estimates share a scale (Cycle 0004). Bounded or direct-mapped counters have hurt (Cycle 0006); keep the counter exact until cost is measured.\n- Insert position: eviction-next placement helped (Cycle 0005); front-of-probation placement hurt (Cycle 0006). Always state the insert position explicitly.\n"
}
```

### 14:35:43 chat

I wrote `cross-cycle-insights.md` in the output directory with the four required sections, in order. It is regenerated from the inlined ledger digest only; I did not read the previous file.

The main points:

- **Recurring wins:** One-shot-insert filtering has the most consistent gains. Probationary-first eviction, the shared-frequency admission gate, and eviction-next placement each improved hit-rate on workload-a.
- **Recurring failures:** Aging policies, bounded victim scans, the protected tier with second-hit promotion, bounded frequency counters, and front-of-probation insertion all failed to gain or regressed.
- **Open questions:** The Winner labels in Cycles 0004 and 0006 don't match the hit-rate evidence. Per-op time and metadata size have never been measured. Gains have only been seen on workload-a.
- **Per-policy notes:** Grouped by mechanism family, since the digest only names AgedLfu and the bucket policy explicitly.


---

- finished: 2026-10-20T14:35:43+00:00
- duration: 15.65s
- status: ok
- input_tokens: 4
- output_tokens: 2861
- cache_read_tokens: 38426
- cache_write_tokens: 11508
- cost_usd: 0.004117
