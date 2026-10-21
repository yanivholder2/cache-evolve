# Cycle 0008 Insight

## Cohort outcome

Three variants were tried from the doorkeeper-free SLRU + sketch incumbent (cycle 0007 v003): an ARC-style adaptive protected share with a ghost list, a small admission window in front of probation, and conservative-update 4-bit packed sketch counters. Only the sketch-precision variant (v003) won, with a marginal hit-rate gain. The adaptive share was noise-level, and the admission window lost slightly.

## Patterns

- Every change was within noise on hit-rate, again on one trace at one cache size. The incumbent looks close to a local optimum on this workload.
- Adaptive segment sizing found almost no trace-phase variance to exploit. The fixed protected share was already near its best, so the ghost list and demotion bookkeeping add complexity for essentially no return.
- Deferring admission through a tiny window did not recover keys that entry-time gating wrongly rejected. The window is too short for bursty keys to be reused before they face the gate. Entry-time gating is close to sufficient here.
- Sketch precision is a minor lever. Cleaner counts helped only slightly. The intended payoff, a smaller sketch, was not measured. Metadata-size and per-op-time have been missing for eight cycles, so the winner may have won only on a tiny hit-rate edge.
- Variant authors in this cycle could not read the overlay source, so these interpretations rest on hypothesis text and fitness vectors only.

## Open questions

- Does the 4-bit packed sketch really halve metadata without hurting hit-rate or per-op-time? That is unknowable until the cost metrics are measured.
- Would a larger window, or a different workload, make deferred admission or adaptive sizing matter?
- Is the missing cost measurement a harness fault?

## Continuation

Stop tuning segment sizing, admission timing and sketch update rules; all are plateaued on this trace. Treat cycle 0008 v003 (compact conservative sketch) as the new incumbent only if the cost metrics can be verified. Before more hypotheses, fix metadata-size and per-op-time measurement and add a second trace or cache size. Then try a genuinely different axis, such as a different eviction signal (recency plus size or cost awareness), or test existing ideas on a scan-heavy or long-reuse trace.
