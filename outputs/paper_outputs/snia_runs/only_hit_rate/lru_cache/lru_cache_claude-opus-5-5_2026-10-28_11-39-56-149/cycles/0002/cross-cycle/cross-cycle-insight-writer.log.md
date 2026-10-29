# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-28T11:53:02+00:00

## timeline

### 11:53:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Soft filtering of one-hit blocks wins: SLRU probation (c1) and W-TinyLFU's relative, tie-admitting gate (c2) were the only Winners. Each cycle's Winner built on the one before (LRU → SLRU → W-TinyLFU over SLRU).\n- Compare a candidate against the current victim and let ties through. This beats any absolute or strict rule for admission and promotion.\n- hm_0 reliably rewards frequency or long-history evidence at both sizes. Gains there are the easiest to get.\n- Gains from segmented and admission designs tend to grow with capacity.\n\n## Recurring failures\n\n- Hard gates or full trust in weak evidence hurt the recency-driven web_0 trace. Examples: strict-greater TinyLFU rejection (c1) and sending ghost hits straight to protected (c2). One return from outside the cache should not displace keys re-referenced while resident.\n- Front segments sized as a fixed fraction of capacity fail at one end of the size range. A narrow 1% window or HIR queue misses web_0's short-range reuse at the small size. A ghost list sized to full capacity over-promotes at the large size.\n- web_0 at the small size has been the weak cell in both cycles. No Winner has fully closed it.\n- Cost axes (`metadata-size`, `per-op-time`) went unmeasured in both cycles. Every claim about cost advantages or overheads is still unverified.\n\n## Open questions\n\n- Can an adaptive window/main split (for example, hill-climbing) fix small-size web_0 without losing the hm_0 gains?\n- How much of the small-size slip comes from sketch tuning (width, reset period, the double increment on miss-then-put) rather than window size?\n- Once cost axes are measured, how do W-TinyLFU's sketch and window, ghost keys, and LIRS links compare with plain SLRU or CLOCK?\n- Can ghost history help if it only feeds the admission comparison and never promotes directly?\n\n## Per-policy notes\n\n- LRU: the baseline. It has been beaten on hit-rate in every cycle.\n- SLRU: a solid base that consistently beats LRU and still underlies the current best. A short probation list loses blocks whose reuse distance is longer than probation, mainly at small capacity.\n- W-TinyLFU over SLRU: the current best. Keep the tie-admit rule. Its window fraction and sketch tuning are the live levers.\n- TinyLFU over plain LRU (strict reject): failed on web_0, worst at the small size. Do not repeat it in that form.\n- CLOCK: drifts toward FIFO on web_0 at large capacity. It is worth keeping only for the cost axes, where its advantages have not been checked.\n- Ghost-to-protected SLRU: hurt. Drop direct promotion.\n- LIRS: beat LRU in most cells but not W-TinyLFU. Its weak cell is the same as W-TinyLFU's (small-size web_0, narrow HIR share). Return to it only for a tuned-HIR test run directly against the current best.\n"
}
```

### 11:53:21 chat

I wrote `cross-cycle-insights.md` in the output dir, using only the ledger digest for cycles 0001 and 0002. It has the four sections in the given order and no numbers.

- **Recurring wins:** filtering blocks seen only once wins when it is soft rather than strict. That covers SLRU in cycle 1 and W-TinyLFU's gate, which compares against the current victim and admits ties, in cycle 2. Each cycle's Winner built on the previous one, from LRU to SLRU to W-TinyLFU over SLRU.
- **Recurring failures:** hard admission gates, and giving full trust to a single return from outside the cache, both hurt web_0. Front segments sized as a fixed fraction of capacity fail at one end of the size range.
- **Still unresolved after both cycles:** web_0 at the small size is the weak spot, and no Winner has closed it. `metadata-size` and `per-op-time` have never been measured, so every cost claim is still unchecked.
- **Open questions:** whether an adaptive window split fixes small-size web_0, how much of the slip comes from sketch tuning, how the policies compare once costs are measured, and whether ghost history helps if it only feeds the admission comparison.
- **Per-policy notes:** short entries for LRU, SLRU, W-TinyLFU over SLRU (the current best), TinyLFU over plain LRU, CLOCK, SLRU with ghost hits sent straight to protected, and LIRS.


---

- finished: 2026-10-28T11:53:21+00:00
- duration: 18.95s
- status: ok
- input_tokens: 4
- output_tokens: 1658
- cache_read_tokens: 36158
- cache_write_tokens: 7392
- cost_usd: 0.099544
