# Cross-Cycle Insights

## Recurring wins

- **Enlarging resident capacity (main queue) beats widening admission.** Transferring slots from the small queue to main was the single most reliable gain; it appeared in cycle 0001 and compounded in cycle 0002. The benefit is workload-agnostic within this trace: more main slots slow eviction churn regardless of what admission logic sits upstream.
- **Ghost precision at main_size/2 is a robust sweet spot.** Halving the ghost cap from its baseline improved hit rate in both cycle 0001 (on the original config) and cycle 0002 (on the enlarged-main config), confirming the gain is not an artifact of any single configuration state. The two levers — larger main, tighter ghost — amplify each other.
- **Stacking complementary structural changes outperforms single-axis tuning.** The cycle 0002 winner combined small-queue shrinkage and ghost compression; its hit rate exceeded any single-axis variant, confirming the levers are additive rather than redundant.

## Recurring failures

- **Loosening admission upstream of main degrades quality faster than second-chance can recover.** Lowering the promotion threshold or expanding the ghost window without a compensating mechanism consistently hurt hit rate (cycle 0001 v002/v003). Second-chance eviction cannot shed lower-quality entries fast enough when the admission gate widens.
- **Further small-queue shrinkage below ~10% yields diminishing returns.** Each incremental slot transferred from small to main after the initial 25%→10% step produced visibly smaller gains (cycle 0002 v001). This axis is effectively exhausted.
- **Ghost compression below main_size/2 discards genuine signal.** The workload's re-use distances fall in the range that requires approximately main_size/2 ghost capacity. Going shorter (cycle 0003 v001) hurt hit rate, confirming the boundary from below after cycle 0001 confirmed it from above.
- **Reducing MAX_FREQ below 3 cuts retention the hot set actually uses.** Tightening second-chance depth (cycle 0003 v003) was clearly harmful; the hot set includes items that legitimately cycle through a third eviction reprieve.

## Open questions

- **Does raising MAX_FREQ above 3 retain additional hot items?** Cycle 0003 established 3 as a lower bound but left the upper direction entirely unprobed. The hot set's temporal stickiness suggests headroom may exist at 4 or 5.
- **Does freq > 0 admission interact positively with a higher MAX_FREQ?** The wider gate is safe but inert alone; pairing it with deeper second-chance retention may give newly admitted items enough time to prove recurrence before eviction.
- **Are there unexplored structural axes?** Queue sizing and simple threshold knobs are well-characterised. Eviction ordering within main (e.g., frequency-weighted rather than FIFO within the second-chance tier) and hit-count reset policy on ghost re-admission have not been tested.
- **Is freq > 0 + current config a cleaner baseline than freq > 1?** The change is confirmed neutral, not harmful; adopting it simplifies future experiments on admission without risk.

## Per-policy notes

- **S3-FIFO — small queue:** Functional range on this workload is roughly 8–10% of total capacity. Below 8% the marginal gain per slot transferred to main approaches zero. The 25%→10% step dominated all later small-queue experiments.
- **S3-FIFO — ghost queue:** Hard floor at main_size/2 confirmed from both directions across three cycles. Re-use distances in the trace fill this window; anything shorter misses genuine re-accesses, anything larger admits noise from distant non-recurring accesses.
- **S3-FIFO — main queue / second-chance:** MAX_FREQ = 3 is a confirmed lower bound. Upper bound (≥ 4) is the highest-priority unexplored knob entering cycle 0004. The second-chance mechanism is a lagging corrector, not a leading filter — it cannot substitute for upstream admission quality.
- **S3-FIFO — promotion threshold:** freq > 1 has been the stable default; freq > 0 is safe and potentially useful as a combined lever but yields no independent hit-rate improvement on this trace given the narrow small queue.
