# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:00:26+00:00

## timeline

### 12:00:42 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Soft one-shot filtering has won all three cycles. Each Winner made it harder for a block seen once to displace blocks that were reused, without ever blocking a block's second access. The line runs SLRU (0001), then W-TinyLFU over SLRU (0002), then a tighter sketch on W-TinyLFU (0003).\n- Relative, lenient admission works. Comparing a candidate against the current victim and admitting ties has beaten absolute gates and full promotion of returning keys.\n- hm_0 (scan-heavy, long reuse distance) keeps rewarding stronger frequency evidence: segmentation, sketches, dedup on miss-then-put, doorkeeper. Most of the cumulative gain has come from hm_0.\n- Each cycle has built on the previous Winner. Incremental lineage has worked better than replacing the policy.\n\n## Recurring failures\n\n- Small-size web_0 has been the weak cell in every cycle and has never been closed. Each lever tried so far moved hit-rate between traces or sizes instead of adding to it.\n- Hard gates and over-weighted out-of-cache evidence have hurt web_0, where reuse is short-range recency. Examples are strict-greater rejection (0001) and ghost hits promoted straight to protected (0002).\n- Fixed fractions tied to capacity fail at one end of the size range. This held for the 1% window and HIR share, the capacity-sized ghost list, and climber step and sample period scaled linearly with capacity (0001–0003).\n- Bundling several changes into one variant has repeatedly made it impossible to say which change caused a loss (0003's four-part sketch change, 0003's coupled LIRS settings).\n- Longer frequency memory and colder first-touch estimates reliably cost web_0. Examples are the longer reset and the doorkeeper.\n- `metadata-size` and `per-op-time` have not been measured in any cycle. Every claim about cost (CLOCK being cheaper, sketch, ghost and LIRS overhead) is still unverified.\n\n## Open questions\n\n- Small-size web_0 looks limited by front-segment (window) size, not sketch noise. Can an adaptive window with steps capped at large capacity fix it without giving back large-size web_0?\n- Which part of the 0003 sketch change causes the web_0 loss: the longer reset, the doorkeeper, or both? Do dedup and a wider sketch alone keep the hm_0 gain?\n- Is there one front-segment share that suits both traces, or does it have to adapt per workload?\n- How do the Winners rank once the cost axes are measured? Does CLOCK-style segmentation hold up there?\n\n## Per-policy notes\n\n- LRU (baseline): beaten in every cycle; only useful as the reference point.\n- SLRU: the base of every Winner. Gains grew with capacity. A short probation list loses small-size web_0 blocks whose reuse distance is longer than probation.\n- CLOCK: drifts toward FIFO on web_0 at large size because its bits stay set. Its expected cost advantage has never been measured. Keep it in reserve for the cost axes.\n- TinyLFU over plain LRU: a hard gate that loses on web_0, worst at small size. Do not retry it.\n- W-TinyLFU over SLRU: the current lineage and the Winner in 0002 and 0003. It is strong on hm_0. Its 1% window is too narrow for small-size web_0. Sketch tuning trades web_0 for hm_0.\n- Ghost lists: useful only as an input to admission decisions. Ghost-to-protected promotion hurts.\n- LIRS: beats LRU but never beat the W-TinyLFU line. HIR-share tuning just trades hm_0 against web_0. Dropped.\n"
}
```

### 12:00:50 chat

I wrote `cross-cycle-insights.md` from the three-cycle ledger digest, using the four fixed sections in order.

- **Recurring wins:** soft filtering of blocks seen only once has won all three cycles. The winning line is SLRU (cycle 0001), then W-TinyLFU over SLRU (0002), then a tighter sketch on W-TinyLFU (0003). Admission that compares a new key with the eviction victim and admits ties keeps working. Most of the gains so far came from hm_0.
- **Recurring failures:** web_0 at the small size has been the weak spot in every cycle, and no change has fixed it. Settings fixed as a share of capacity keep failing at one end of the size range. Hard admission gates have hurt web_0. Variants that change several things at once can't be diagnosed. `metadata-size` and `per-op-time` have never been measured, so no cost claim has been checked.
- **Open questions:** whether an adaptive window, with its step capped at large capacity, closes the small-size web_0 gap. Which part of cycle 0003's sketch change causes its web_0 loss. Whether one window size suits both traces. How the winners rank once cost is measured.
- **Per-policy notes:** one short entry each for LRU, SLRU, CLOCK, TinyLFU, W-TinyLFU, ghost lists and LIRS. TinyLFU over plain LRU and LIRS are marked as dropped.


---

- finished: 2026-10-28T12:00:50+00:00
- duration: 24.22s
- status: ok
- input_tokens: 4
- output_tokens: 1867
- cache_read_tokens: 37162
- cache_write_tokens: 8620
- cost_usd: 0.113748
