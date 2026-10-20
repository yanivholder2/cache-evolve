## Recurring wins

- No variant has shown a demonstrated hit-rate gain across cycles 0001–0005. Every "Winner" so far has been neutral against its parent.
- Behavior-preserving changes (cost refactors, lookup restructuring, bitfield packing) consistently stayed flat on hit-rate. That is a safety signal, not a performance gain.
- The two-hit promotion bar has held up: relaxing it to one hit lost hits (0001), and raising it to three was neutral (0003). Nothing has beaten it.

## Recurring failures

- Ghost-related changes have lost hits in every direction tried: widening the ghost window (0003), and routing ghost-hit re-admissions through the small queue (0004). The ghost-admission lever has not produced a gain.
- Adaptive small-queue sizing driven by ghost hits lost hits (0005). Fixed-split sizing and ghost-window sizing have both failed to improve on the baseline.
- Relaxing the promotion bar to one hit lost hits (0001). Further promotion-bar tweaks have been flat or negative.
- Frequency-axis changes (main-queue cap in 0002, decay step in 0005) were flat both times. The axis looks inert on this trace.
- Small-queue ratio tweaks (0001) were flat, so the sizing axis has shown no signal at this capacity.

## Open questions

- Cost has never been reported. Metadata-size and per-op-time are missing from every Winner across five cycles, so no neutral hit-rate result can be ranked against the baseline. Does any cost-side change actually help?
- Overlay verification is unresolved. Edits were unconfirmed or not inspectable in 0002 (v003 absent), 0003 (v002 `freq > 2` unconfirmed), 0004 (v001 and v003), and 0005. Are the flat results attributable to the intended changes?
- Is the 0002 v003 hit-rate drop real or a build artifact? It has not been reproduced on a clean build.
- Does the picture change at other capacities or workloads? All evidence is one trace (lirs workload-a) at one capacity (2048), so the flat and negative results may reflect trace insensitivity rather than the policy.
- Would a larger or damped sizing change, rather than a ±1 step, move hit-rate?

## Per-policy notes

- S3-FIFO (the only policy in the ledger):
  - Promotion bar: two hits is the best-supported setting so far on this trace.
  - Ghost handling: window widening and small-queue routing of ghost hits both lost hits. Treat ghost admission as a sensitive, unresolved area.
  - Frequency and sizing: largely inert on this trace, with adaptive sizing as the one tested variant that hurt.
  - Cost: unmeasured for all variants. Cost-side candidates (resident-key probe changes, ghost-lookup refactor, bitfield packing) need per-op-time and metadata-size before they can be called wins.
