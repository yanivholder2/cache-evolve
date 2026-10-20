## Recurring wins

- Filtering or deprioritising one-shot inserts is the only direction that has produced hit-rate gains more than once (Cycles 0003, 0004, 0005, 0007). Gains came from probationary-first eviction, the shared-frequency admission gate, eviction-next insert position, and the second-miss ghost list.
- Scan resistance is the main lever. Changes that alter which entries are displaced, rather than how the existing ranking is computed, are the ones that moved hit-rate positively.
- Strict comparison in the shared-frequency admission predicate is the better default (Cycle 0007 tie-admission gave no benefit).

## Recurring failures

- Placing newly admitted keys at the front or victim end of probation has lost hit-rate repeatedly (Cycles 0006, 0007, 0008). Eviction-next placement behind the admission gate also regressed (Cycle 0009), so making admitted keys eviction-eligible sooner has not helped on this workload.
- Protected-tier designs have not paid off (Cycles 0004, 0005). Shrinking protected capacity changed nothing (Cycle 0006), so the cause is more likely the demotion or second-hit rule than capacity.
- Pruning or zeroing singleton frequency history loses hit-rate under either trigger (Cycles 0008, 0009).
- Widening the ghost-list window regresses (Cycle 0008). Stacking a first-miss refusal in front of the counter gate also regresses (Cycle 0009), since the counter gate already bypasses one-shot keys.
- Bounding frequency metadata with aliasing, u8 saturation, or direct mapping cost hit-rate (Cycle 0006). Exact-key or one-change-at-a-time variants have not been tested alone.
- Hit-rate-only geometry and counter-width tweaks were flat (Cycles 0001, 0002). Bounded victim scans were flat (Cycles 0003, 0004), so exact-minimum eviction buys little, but this is a null on hit-rate, not a demonstrated cost saving.

## Open questions

- The recorded Winner conflicts with its own hit-rate evidence in Cycles 0004, 0006, 0007, 0008, and 0009. The selection rule needs checking before any Winner is used as a precedent.
- Metadata size and per-op time have never been measured in any variant (Cycles 0001 to 0009). Every cost-motivated hypothesis remains unverified.
- Several insights could not confirm that their overlay was active under the harness (Cycles 0003, 0004, 0005, 0009). Flat results cannot yet be read as null results about the mechanism.
- Nearly all gains rest on workload-a. Reproduction on a second workload is still missing for the shared-frequency gate, eviction-next placement, and the ghost list.
- The ghost-list gain (Cycle 0007) has not been reproduced under the same window. Its robustness is unknown.
- Does eviction-next placement (Cycle 0005 gain) mean the same thing as victim-end or front placement (later losses)? Placement semantics need to be pinned down.
- Does the demotion or second-hit rule explain the protected-tier failures?

## Per-policy notes

- **Fixed-width buckets (Cycle 0001):** the only winner had a marginal hit-rate gain and an unmeasured per-op scan cost.
- **Bucket static dispatch (Cycle 0001):** no hit-rate movement; latency effect untested.
- **Fingerprint filter (Cycle 0002):** flat hit-rate, yet recorded as Winner. Per-op savings untested.
- **AgedLfu:** aging and counter-width changes were flat (Cycles 0001, 0002). Whether halving fires under the harness is unknown. Zero-pruning during halving regressed (Cycle 0009). Halving without removing entries is untested.
- **Probationary / protected two-region:** probationary-first eviction gained (Cycle 0003). Protected tiers lost (Cycles 0004, 0005). Shrinking the protected region was flat (Cycle 0006).
- **Shared-frequency admission gate:** the most consistent positive signal (Cycles 0004, 0005). Tie-admission and a stacked first-miss refusal did not help (Cycles 0007, 0009). The shared counter's metadata cost is unmeasured.
- **Ghost list / second-miss admission:** one-capacity window gained once (Cycle 0007). Widening it regressed (Cycle 0008).
- **Insert position:** eviction-next gained once (Cycle 0005) and regressed later (Cycle 0009). Victim-end and front placement lost (Cycles 0006, 0007, 0008).
- **Bounded victim scan:** flat in two cycles. Per-op benefit unmeasured.
- **Direct-mapped or saturating frequency store:** lost hit-rate (Cycle 0006). Cause unresolved across three simultaneous changes.
