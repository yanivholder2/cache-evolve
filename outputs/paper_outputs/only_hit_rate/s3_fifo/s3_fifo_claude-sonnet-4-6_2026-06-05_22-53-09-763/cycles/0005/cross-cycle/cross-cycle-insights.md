## Recurring wins

- **Shifting capacity from small to main queue.** Reducing the small-queue ratio from 25% → 10% was the single largest hit-rate gain across all cycles. The benefit came from giving the main queue more resident slots, not from loosening any admission gate.
- **Ghost window precision at main_size/2.** Halving the ghost cap from the full main_size baseline improved hit rate independently of small-queue sizing, and the gain was complementary when both changes were stacked. The workload's genuine re-use distances sit comfortably inside a main_size/2 window.
- **Stacking independent wins compounds.** The cycle-0002 composite (small=10% + ghost=main_size/2) outperformed either change alone, confirming that these two levers address different bottlenecks and do not cannibalize each other.
- **Raising the MAX_FREQ ceiling incrementally helps.** Each step upward (3→4, 4→5) yielded a real but diminishing positive: the hot set contains a tail of items that legitimately earn and use extra retention passes. Gains are now small enough that this axis is nearly exhausted.
- **Move-to-front within main is the primary recency mechanism.** Revealed by the regression when it was replaced in cycle 0005: unconditional repositioning of re-accessed items keeps them away from the eviction frontier, and this implicit recency sort is load-bearing for hit rate.

## Recurring failures

- **Relaxing the promotion threshold (freq > 1 → freq > 0).** Tried solo (neutral, cycle 0003), in combination with ghost precision (harmful, cycle 0004), and the result never improves. The narrow small queue ensures that single-access traffic is infrequent enough that wider admission only adds churn without meaningful new signal.
- **Extending the ghost window above main_size/2.** Tried in cycle 0001 and again in cycle 0003; both times the larger window admitted re-accesses from genuinely cold, non-recurring items, degrading the signal quality that ghost tracking is supposed to provide.
- **Ghost re-admission credit seeding (freq > 0 on re-entry).** Giving ghost-promoted items a head-start frequency displaces incumbents with earned residency and consistently hurts. The ghost signal validates re-use potential but does not justify overriding the main queue's organic priority order.
- **CLOCK-style eviction scan in main.** Removing the implicit move-to-front repositioning caused a large regression; frequency-only eviction without recency reordering cannot maintain hot-item residency on this workload.
- **Further small-queue shrinkage below 10%.** The marginal gain per percentage point transferred to main collapses sharply past the 10% point; this axis is effectively exhausted.

## Open questions

- **Conditional or selective move-to-front.** Since unconditional repositioning is load-bearing but blunt, could repositioning gated on a frequency threshold (e.g., only reposition items with freq ≥ 2) squeeze more out of eviction ordering without discarding the recency benefit?
- **MAX_FREQ upper bound.** Returns from 3→4 and 4→5 are each smaller than the last; is there a useful stopping point before gains become noise-level, and does the hot-set frequency distribution actually have a meaningful tail above 5?
- **Adaptive queue sizing under phase shifts.** All tuning has been against a single trace regime. Whether the optimal small-queue ratio and ghost cap hold across scan-heavy or bursty phases remains entirely untested.
- **Ghost re-admission at freq=0 with higher MAX_FREQ.** Re-admitting without a credit (freq=0) combined with the now-higher retention ceiling is a structurally different combination from anything tried; the extended ceiling gives newly admitted items time to prove value that the old ceiling did not.

## Per-policy notes

- **S3-FIFO (the only policy studied):** The three foundational parameters — small-queue ratio, ghost-window size, and MAX_FREQ ceiling — have been mapped to near-optimal values: small≈10%, ghost≈main_size/2, MAX_FREQ=5. The admission threshold (freq > 1) is correct and robust; relaxing it is never beneficial. The most sensitive unexplored lever is eviction *ordering* within the main queue: move-to-front is the mechanism through which recency information enters the cache, and conditional variants of it are the highest-yield remaining direction.
