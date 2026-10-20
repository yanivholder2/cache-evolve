# Cross-cycle insights

## Recurring wins

- No variant has shown a demonstrated hit-rate gain over the parent. "Winner" has consistently meant "did not regress."
- Behavior-preserving cost refactors (resident-key probes, ghost-lookup restructuring, membership bit packing, early return on resident hits) have repeatedly stayed hit-rate neutral, as intended. They are the only lever family that reliably passes the gate, but none has been shown to reduce cost.
- The two-hit small-queue promotion bar is worth keeping. Relaxing it lost hits, and the bar has not been beaten by raising it.

## Recurring failures

- Main-queue frequency tuning is inert. Raising the frequency cap and halving the decay step both came back flat, so frequency-axis tweaks have shown no signal.
- Ghost-path changes have lost hits in both directions. Widening the ghost window lost hits, and routing ghost-hit re-admissions through the small queue also lost hits. Ghost admission and window size are not yet a source of gains.
- Adaptive small-queue sizing driven by ghost hits lost hits. The ghost signal appears too noisy to steer the split. Fixed-size small-queue tweaks were also neutral.
- Main-queue retention changes lost hits. Keeping warm residents longer by changing reinsert position displaced value rather than adding it.
- Promotion-bar tweaks are saturated. Relaxing lost hits; raising was neutral.
- Cost is never measured. No cycle has reported metadata-size or per-op-time, so no cost-side candidate can be ranked against the baseline.
- Overlay verification keeps failing. Several variants had edits that were absent, not confirmed, or not inspected, so their results cannot be tied to the intended change.

## Open questions

- Do the cost-side candidates (resident-key probe variants, ghost-lookup refactor, membership bit packing, early return) reduce per-op-time or metadata-size? A rerun reporting both is needed before any is called a win.
- Did the intended edits land in the built code for the variants whose overlays were not confirmed or not inspected? Until verified, flat or negative results say little about the intended changes.
- Was the layout variant's hit-rate drop real, or an artifact of the mismatched overlay? It needs a clean-build rerun.
- Is the insensitivity of frequency, sizing, and promotion levers a property of this trace, or of the single capacity? Results come from one trace and one capacity only.
- Would ghost re-admissions behave differently under a different promotion bar, or if ghost hits were admitted by a different path?
- Does the comparison baseline match the intended parent for each variant? One variant cited another variant as its control, so attribution needs confirmation.

## Per-policy notes

- S3-FIFO, small queue: the promotion bar is the most consistently informative knob. Relaxing it lost hits and raising it was flat. Queue sizing, fixed or adaptive, has not produced gains.
- S3-FIFO, main queue: frequency cap and decay changes were flat. Reinsert position and retention changes lost hits. The current head reinsert appears to be leaving no obvious value on the table on this trace.
- S3-FIFO, ghost queue: window widening and small-queue re-routing of ghost hits both lost hits. Any future ghost-size work must also change how ghost hits are admitted, and should be tested on more than one trace or capacity.
- S3-FIFO, membership and lookup structures: restructuring that preserves eviction order has stayed hit-rate neutral. Its benefit is untested because cost metrics are missing.
- Evidence base for all of the above: one trace (lirs workload-a) at one capacity (2048).
