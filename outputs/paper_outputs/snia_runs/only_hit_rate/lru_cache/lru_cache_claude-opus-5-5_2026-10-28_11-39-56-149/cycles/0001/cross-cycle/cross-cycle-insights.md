# Cross-Cycle Insights

_Projection of 1 completed cycle (0001). Patterns below are provisional until later cycles confirm them._

## Recurring wins

- Soft scan resistance pays off. A one-time block is held in a probation area or given an unset reference bit instead of being rejected outright. SLRU (probation + protected) was the only cohort Winner. It improved hit-rate on every trace and size.
- Segmentation gains grow with capacity. More room lets the protected segment hold the frequently reused working set.
- Frequency-aware filtering helps consistently on the frequency-skewed trace (hm_0) at all sizes.

## Recurring failures

- Hard admission gates hurt recency-driven workloads. Plain TinyLFU in front of LRU, with a strict-greater reject on count ties, lost on web_0, worst at small capacity. Any mechanism that delays or blocks a block's second access loses short-range reuse.
- A plain CLOCK ring at large capacity decays toward FIFO because its reference bits stay set. This produced a slight web_0 loss.
- Every fitness vector so far has held hit-rate only. `metadata-size` and `per-op-time` were never measured, so no cost-axis claim has been verified.

## Open questions

- Does SLRU keep its edge once `metadata-size` and `per-op-time` are measured? Does CLOCK's expected cost advantage make up for its small hit-rate deficit?
- Would a larger or adaptive probation share at small capacity recover blocks whose reuse distance is longer than probation?
- Can admission filtering help recency-heavy traces (web_0)? Candidate designs: W-TinyLFU, which puts an admission-free recency window in front of SLRU, or a sketch that admits low-count ties.
- Can CLOCK-based segmentation keep CLOCK's cheap `get` and still match SLRU's hit-rate?

## Per-policy notes

- **SLRU:** Current best lineage. The next step is to tune the probation/protected split, especially at small size, or try an adaptive split.
- **CLOCK:** Kept as an option for the cost axes; its hit-rate is roughly LRU-level. Its large-ring degeneration on web_0 needs a fix, such as segmentation or a bit-decay variant.
- **TinyLFU (over plain LRU):** Do not retry the v003 form. Use the sketch only behind a recency window, with lenient tie admission.
- **Traces:** hm_0 rewards frequency filtering. web_0 rewards short-range recency, most strongly at small capacity. A good policy has to serve both.
