## Recurring wins

- Soft filtering of one-hit blocks wins: SLRU probation (c1) and W-TinyLFU's relative, tie-admitting gate (c2) were the only Winners. Each cycle's Winner built on the one before (LRU → SLRU → W-TinyLFU over SLRU).
- Compare a candidate against the current victim and let ties through. This beats any absolute or strict rule for admission and promotion.
- hm_0 reliably rewards frequency or long-history evidence at both sizes. Gains there are the easiest to get.
- Gains from segmented and admission designs tend to grow with capacity.

## Recurring failures

- Hard gates or full trust in weak evidence hurt the recency-driven web_0 trace. Examples: strict-greater TinyLFU rejection (c1) and sending ghost hits straight to protected (c2). One return from outside the cache should not displace keys re-referenced while resident.
- Front segments sized as a fixed fraction of capacity fail at one end of the size range. A narrow 1% window or HIR queue misses web_0's short-range reuse at the small size. A ghost list sized to full capacity over-promotes at the large size.
- web_0 at the small size has been the weak cell in both cycles. No Winner has fully closed it.
- Cost axes (`metadata-size`, `per-op-time`) went unmeasured in both cycles. Every claim about cost advantages or overheads is still unverified.

## Open questions

- Can an adaptive window/main split (for example, hill-climbing) fix small-size web_0 without losing the hm_0 gains?
- How much of the small-size slip comes from sketch tuning (width, reset period, the double increment on miss-then-put) rather than window size?
- Once cost axes are measured, how do W-TinyLFU's sketch and window, ghost keys, and LIRS links compare with plain SLRU or CLOCK?
- Can ghost history help if it only feeds the admission comparison and never promotes directly?

## Per-policy notes

- LRU: the baseline. It has been beaten on hit-rate in every cycle.
- SLRU: a solid base that consistently beats LRU and still underlies the current best. A short probation list loses blocks whose reuse distance is longer than probation, mainly at small capacity.
- W-TinyLFU over SLRU: the current best. Keep the tie-admit rule. Its window fraction and sketch tuning are the live levers.
- TinyLFU over plain LRU (strict reject): failed on web_0, worst at the small size. Do not repeat it in that form.
- CLOCK: drifts toward FIFO on web_0 at large capacity. It is worth keeping only for the cost axes, where its advantages have not been checked.
- Ghost-to-protected SLRU: hurt. Drop direct promotion.
- LIRS: beat LRU in most cells but not W-TinyLFU. Its weak cell is the same as W-TinyLFU's (small-size web_0, narrow HIR share). Return to it only for a tuned-HIR test run directly against the current best.
