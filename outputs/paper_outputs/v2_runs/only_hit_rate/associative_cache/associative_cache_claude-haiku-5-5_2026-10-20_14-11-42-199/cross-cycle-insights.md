## Recurring wins

- Shared-frequency admission or victim comparison, which keeps one-shot keys from displacing keys with proven reuse, has produced the only consistent hit-rate gains (Cycles 0004, 0005, 0010). Magnitudes are small and single-trace.
- Scan-resistant probationary-first eviction is the other repeated gain source (Cycle 0003), and it is the same family as the admission gate.
- Recurrence-gated admission through a one-capacity ghost list showed a gain once (Cycle 0007). It has not been retested since, so it is a lead, not a win.

## Recurring failures

- Changing where newly admitted keys sit in the eviction order has lost hit-rate repeatedly: front of probation (Cycle 0006), victim end (Cycles 0007, 0008), and eviction-next stacked on the gate (Cycle 0009). Only one eviction-next result (Cycle 0005) was positive, and its placement semantics are not confirmed to match.
- Aging by halving or pruning counters has lost hit-rate in every form tried: zero-pruning (Cycle 0009), plain floor-halving (Cycle 0010), and singleton pruning (Cycle 0008). Small counter magnitudes erase the one-hit versus no-hit distinction the gate depends on.
- Protected-tier and second-hit variants have not held up (Cycles 0004 to 0006). Shrinking the protected region changed nothing, and removing forced demotion did not recover the loss.
- Loosening the admission comparison, whether by ties, by bypassing the counter check, or by stacking a recurrence refusal in front of the gate, lowers hit-rate (Cycles 0007, 0009, 0010).
- Widening the ghost-list window (Cycle 0008) and bounding the victim scan as a hit-rate lever (Cycles 0003, 0004) gave no gain.
- Hit-rate-only geometry tweaks (Cycle 0001) produced at most marginal gains with no cost evidence.

## Open questions

- Winner labels still conflict with the recorded evidence in several cycles (0004, 0006 to 0009). Cycle 0010 is the first where the label matches the data. The selection rule should be checked before any Winner is used as a precedent.
- Cost is unmeasured in every cycle. Metadata size and per-op time are missing for the shared counter, the ghost list, the per-miss probationary scan, and the full halving pass. The cost side of every hypothesis is still unverified.
- Does the Cycle 0010 victim-by-frequency gain hold on a second trace and cache size, and does its extra per-miss scan cost offset it?
- Does the Cycle 0007 recurrence gain survive when the counter check is kept on the ghost path, as Cycle 0010 suggests?
- Is the protected-tier failure caused by the second-hit signal or by the demotion rule?
- Is the eviction-next gain of Cycle 0005 a different mechanism from the victim-end and front-placement losses?

## Per-policy notes

- Shared-frequency admission / victim selection (LFU-like): the most consistent positive direction. Keep the comparison strict, and test extra cost before accepting it.
- Aging and counter halving (AgedLfu-style): negative in every variant so far. Do not retry removal or floor-based halving without keeping previously seen counts at or above 1.
- Two-region / protected tier (SLRU-like): repeated failures. Capacity is not the lever; the second-hit or demotion rule is the open suspect.
- Ghost-list / recurrence gating: one positive result (Cycle 0007), with mixed results when stacked or overridden. Keep the one-capacity window and the counter check as the base for any retest.
- Insert position (MRU, victim end, eviction-next, front of probation): placement changes that make new keys eviction-eligible sooner have mostly lost. State the insert position explicitly in every hypothesis.
- Bucket geometry and fixed-width buckets (Cycle 0001): no cost-side evidence; treat as unexplored until paired with per-op time.
