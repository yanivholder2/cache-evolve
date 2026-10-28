## Recurring wins

- Soft one-shot filtering has won all three cycles. Each Winner made it harder for a block seen once to displace blocks that were reused, without ever blocking a block's second access. The line runs SLRU (0001), then W-TinyLFU over SLRU (0002), then a tighter sketch on W-TinyLFU (0003).
- Relative, lenient admission works. Comparing a candidate against the current victim and admitting ties has beaten absolute gates and full promotion of returning keys.
- hm_0 (scan-heavy, long reuse distance) keeps rewarding stronger frequency evidence: segmentation, sketches, dedup on miss-then-put, doorkeeper. Most of the cumulative gain has come from hm_0.
- Each cycle has built on the previous Winner. Incremental lineage has worked better than replacing the policy.

## Recurring failures

- Small-size web_0 has been the weak cell in every cycle and has never been closed. Each lever tried so far moved hit-rate between traces or sizes instead of adding to it.
- Hard gates and over-weighted out-of-cache evidence have hurt web_0, where reuse is short-range recency. Examples are strict-greater rejection (0001) and ghost hits promoted straight to protected (0002).
- Fixed fractions tied to capacity fail at one end of the size range. This held for the 1% window and HIR share, the capacity-sized ghost list, and climber step and sample period scaled linearly with capacity (0001–0003).
- Bundling several changes into one variant has repeatedly made it impossible to say which change caused a loss (0003's four-part sketch change, 0003's coupled LIRS settings).
- Longer frequency memory and colder first-touch estimates reliably cost web_0. Examples are the longer reset and the doorkeeper.
- `metadata-size` and `per-op-time` have not been measured in any cycle. Every claim about cost (CLOCK being cheaper, sketch, ghost and LIRS overhead) is still unverified.

## Open questions

- Small-size web_0 looks limited by front-segment (window) size, not sketch noise. Can an adaptive window with steps capped at large capacity fix it without giving back large-size web_0?
- Which part of the 0003 sketch change causes the web_0 loss: the longer reset, the doorkeeper, or both? Do dedup and a wider sketch alone keep the hm_0 gain?
- Is there one front-segment share that suits both traces, or does it have to adapt per workload?
- How do the Winners rank once the cost axes are measured? Does CLOCK-style segmentation hold up there?

## Per-policy notes

- LRU (baseline): beaten in every cycle; only useful as the reference point.
- SLRU: the base of every Winner. Gains grew with capacity. A short probation list loses small-size web_0 blocks whose reuse distance is longer than probation.
- CLOCK: drifts toward FIFO on web_0 at large size because its bits stay set. Its expected cost advantage has never been measured. Keep it in reserve for the cost axes.
- TinyLFU over plain LRU: a hard gate that loses on web_0, worst at small size. Do not retry it.
- W-TinyLFU over SLRU: the current lineage and the Winner in 0002 and 0003. It is strong on hm_0. Its 1% window is too narrow for small-size web_0. Sketch tuning trades web_0 for hm_0.
- Ghost lists: useful only as an input to admission decisions. Ghost-to-protected promotion hurts.
- LIRS: beats LRU but never beat the W-TinyLFU line. HIR-share tuning just trades hm_0 against web_0. Dropped.
