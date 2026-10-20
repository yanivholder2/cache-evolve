## Recurring wins

- Filtering one-shot inserts so they do not displace entries with proven reuse is the only direction that has produced hit-rate gains more than once. It shows up as a shared-frequency admission gate, an eviction-next insert position, and a recurrence (second-miss) gate.
- Strict frequency comparison in the shared-frequency admission gate has been the more reliable form. Admitting on ties gave no benefit.
- Probationary-first eviction has held up as a base for the scan-resistance gains.

## Recurring failures

- Placing admitted keys at the probationary victim end lost hit-rate in two separate cycles.
- Protected-tier variants (second-hit promotion, demote-on-promotion, and demote-skip) lost or stayed below parent in several cycles. Shrinking the protected region changed nothing, so the failure seems to lie in the demotion or second-hit signal, not in capacity.
- Widening the second-miss ghost window regressed, which suggests stale one-shot keys are admitted when the first-miss memory is longer.
- Compact or bounded frequency stores (direct-mapped, saturating, u8 counters, singleton pruning) reduced hit-rate or gave small regressions. The cause is confounded across changes.
- Hit-rate-only geometry and aging-period tweaks (bucket repartitioning, the AgedLfu period, aging counter width, fingerprint filter, sampled victim scan) were flat or marginal. None was paired with a cost measurement.

## Open questions

- The recorded Winner often does not match the hit-rate evidence in its own insight (Cycles 0004, 0006, 0007, 0008). The selection rule needs checking before any Winner is used as precedent.
- Per-op time and metadata size have not been measured in any cycle. Every cost-motivated hypothesis remains unverified.
- Do the admission gate and eviction-next insert position stack, or are they redundant? Do they change the same victims?
- Does the ghost-list recurrence gain (Cycle 0007) reproduce on a second workload or with the same setup?
- Does the shared-frequency gain (Cycle 0004) hold beyond workload-a?
- Were overlays active under the harness in several cycles? Flat results cannot be read as null without that check.

## Per-policy notes

- **Aging LFU:** No hit-rate gain in Cycles 0001 and 0002. Whether the evaluation selects the aging policy at all is unresolved, so do not iterate on its period or counter width alone.
- **Shared-frequency admission gate:** The most consistent positive lever for one-shot filtering. Results depend on a common scale for incoming and resident keys and on unbounded or exact counters; bounding the counter cost hit-rate.
- **Recurrence (ghost-list) gating:** One-capacity window showed a gain (Cycle 0007). Longer windows regressed. Metadata cost per key is not yet measured.
- **Probationary/protected (segmented) structure:** Probationary-first eviction helped once. Protected-tier variants and protected-capacity changes did not. Demotion and second-hit signal are the remaining suspects.
- **Bucket / set geometry and sampled victim scans:** Hit-rate-neutral in every case. Any gain here would have to come from per-op time, which has not been measured.
