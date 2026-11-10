# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:13:10+00:00

## timeline

### 10:13:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Per-key evidence beats global signals. Gating on each key's own ghost-time freq (c4), then on its own ghost age (c5), are the only policy changes that clearly improved on their parent. Global rate or occupancy proxies never managed that.\n- Strict admission into a full main holds up. The two-reuse rule plus freq reset has survived every threshold, split, gate and credit challenge (c1–c3), and later gains came only from refining how ghost returnees route around it.\n- About one small-queue turnover is a confirmed local optimum for the freq-0 ghost-age window. Both a 2× window (c6) and a ½× window (c7) lost.\n- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) consistently keep hit/miss behaviour bit-identical, and an oracle test enforces this. They are safe to port onto any policy parent.\n- Mechanisms whose windows scale with `small_size` can move cells at the large cache size. Fixed-constant changes have only ever helped at the small size.\n\n## Recurring failures\n\n- Single-constant tweaks to the threshold or the small/main split split the two traces: hm_0 gains and web_0 loses, and the effect flips sign across cache sizes. Results stay near noise (c1–c2).\n- Global adaptive signals don't work:\n  - Main occupancy saturates, so a gate on it falls back to the baseline rule.\n  - A ghost-return rate is inflated by the ghost being larger than the small queue, and it fired on the wrong traces.\n  - An unweighted ARC-style split drifts toward a larger small queue (c2–c3).\n- Extra protection in a full main always evicts someone useful. Flat ghost-return credit (c3) and a protected lap for freq-saturated keys (c7) both lost, the latter by putting entries ahead of valuable quick freq-0 returnees.\n- Re-admitting keys to small displaces small residents, which costs more than the main pollution it avoids, especially at the small size. This covers relaps, whether they reset freq (c5) or keep the freq-1 signal (c6, where they duplicated the ghost path).\n- Ghost-path routing tuning is exhausted: windows, ceilings on freq ≥ 1 returnees, and stacked reuse rewards (c6–c7).\n- New producers of ghost entries that share the ghost FIFO or the stamp clock shrink the age gate's effective window and hurt the small size (c7).\n- Per-op-time and metadata-size have never appeared in the fitness vector, for seven cycles running. Several cost-only variants remain unmeasured, and hypotheses repeatedly assumed these dimensions were already being reported.\n\n## Open questions\n\n- What limits web_0 at the large size? It resisted every ghost/small change and moved only marginally when main victims were touched. First-admission filtering of never-seen keys and the small/main split at the large size remain unprobed.\n- If main-victim ghosts are decoupled (with their own budget, or without advancing stamps), do c7's large-size gains survive without the small-size loss?\n- How much do the cost changes actually save on per-op-time and metadata-size, and does alignment cancel the ghost-deque saving on 8-byte keys?\n- Would a gate that directly compares the misses avoided by promoting against the main hits lost to displacement beat the per-key heuristics?\n\n## Per-policy notes\n\n- S3-FIFO baseline (strict `freq > 1` graduation, freq reset): a robust default, and the threshold is not worth changing again.\n- Small/main split: static and adaptive variants both land near noise. Untested only at the large size as a lever for web_0.\n- Ghost path: the current best is the freq-gated plus age-gated (1× turnover) routing of returnees, with quick freq-0 returns going straight to main. Treat it as near-final, and keep any layered mechanism from erasing its per-key freq or age signal.\n- Main eviction: second chance already rewards keys that keep getting hit. Admission boosts add little and hurt quick returnees. Re-qualifying main victims is a small lever and needs to be decoupled from the ghost.\n- Cost line: refactors stay behaviour-identical and should be merged onto the current policy parent. Their value is unknown until the cost dimensions are measured.\n- Trace character: hm_0 rewards real reuse that gets squeezed out of small, especially at the small size. web_0 punishes late one-off rereferences and keys reused exactly twice. Nearly all gains are small and concentrated at the small cache size.\n"
}
```

### 10:13:35 thinking



### 10:13:35 chat

I wrote `cross-cycle-insights.md` in the cycle 0007 cross-cycle output dir. It uses the four fixed sections in order and is built only from the seven-cycle ledger.

- **Recurring wins:** Changes based on each key's own evidence are the only ones that clearly beat their parent. Gating on a key's freq when it went to ghost (cycle 4) and on its ghost age (cycle 5) both did. The strict two-reuse rule with freq reset has held up against every challenge. A ghost-age window of one small-queue turnover is a confirmed local optimum, since both 2× and ½× lost. The cost-only refactors never change hit/miss behaviour.
- **Recurring failures:**
  - Changing one constant (threshold or small/main split) helps one trace and hurts the other.
  - Global signals like main occupancy, ghost-return rate and an unweighted adaptive split don't work.
  - Extra protection inside a full main, and re-admitting keys to small, push out useful entries.
  - New ghost entries that share the ghost queue or its clock shrink the age window.
  - Per-op-time and metadata-size have been missing from the evaluation for all seven cycles.
- **Open questions:** What limits web_0 at the large cache size? Nothing has moved it except a marginal shift in cycle 7, and first-admission filtering and the split at that size are still untried. The file also asks whether cycle 7's main-victim change works once it gets its own ghost budget, and how much the cost refactors actually save.
- **Per-policy notes:** These cover the baseline graduation rule, the small/main split, ghost-path routing (treated as near-final), main eviction, the cost line, and how the two traces differ. Nearly all gains so far are small and at the small cache size.


---

- finished: 2026-11-10T10:13:36+00:00
- duration: 25.24s
- status: ok
- input_tokens: 4
- output_tokens: 2247
- cache_read_tokens: 40792
- cache_write_tokens: 12489
- cost_usd: 0.153821
