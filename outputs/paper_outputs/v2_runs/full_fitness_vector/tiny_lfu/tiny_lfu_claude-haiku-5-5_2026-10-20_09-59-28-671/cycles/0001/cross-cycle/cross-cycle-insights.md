## Recurring wins

- Constant-factor guards that leave eviction order untouched and only skip pointer work on repeated accesses to the newest key give a safe speed gain with no hit-rate cost (cycle 0001, v002). Only one cycle so far, so treat as promising, not established.
- The frequency-based admission gate already does the protective work; eviction-order changes layered on top have not added value.

## Recurring failures

- Rotating rejected admission victims to the MRU tail lowered hit rate and increased per-op time (cycle 0001, v001). Recency rotation duplicates protection the admission gate already provides, and it adds cost. Drop as a direction.
- Segmentation of the resident list (probation/protected) moved hit rate only slightly while adding per-node and per-cache metadata (cycle 0001, v003). Its cost-benefit is weak unless the gain holds elsewhere.

## Open questions

- Does the segmentation gain hold on traces other than lirs workload-a, or is it trace-specific?
- Is the latency drop seen with segmentation real, or measurement variance? Its added bookkeeping does not explain it.
- How much of the constant-factor guard's saving depends on how often accesses hit the newest key? Would it help on workloads with few such repeats?
- All evidence so far comes from a single trace (lirs workload-a); no result has been shown to generalize.

## Per-policy notes

- TinyLFU (admission gate on frequency): the admission gate is the component doing useful work; keep it as the base and avoid adding recency-based victim handling on top.
- Segmented or probation/protected variants: only tested as a segmentation of the resident list; hit-rate effect is small and needs a second trace before further investment.
- Recency-rotation variants of rejected victims: negative on both hit rate and cost; not worth revisiting in this form.
