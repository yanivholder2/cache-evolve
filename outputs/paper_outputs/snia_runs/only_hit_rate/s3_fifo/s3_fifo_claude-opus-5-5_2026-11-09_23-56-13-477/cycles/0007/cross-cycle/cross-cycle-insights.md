## Recurring wins

- Per-key evidence beats global signals. Gating on each key's own ghost-time freq (c4), then on its own ghost age (c5), are the only policy changes that clearly improved on their parent. Global rate or occupancy proxies never managed that.
- Strict admission into a full main holds up. The two-reuse rule plus freq reset has survived every threshold, split, gate and credit challenge (c1–c3), and later gains came only from refining how ghost returnees route around it.
- About one small-queue turnover is a confirmed local optimum for the freq-0 ghost-age window. Both a 2× window (c6) and a ½× window (c7) lost.
- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) consistently keep hit/miss behaviour bit-identical, and an oracle test enforces this. They are safe to port onto any policy parent.
- Mechanisms whose windows scale with `small_size` can move cells at the large cache size. Fixed-constant changes have only ever helped at the small size.

## Recurring failures

- Single-constant tweaks to the threshold or the small/main split split the two traces: hm_0 gains and web_0 loses, and the effect flips sign across cache sizes. Results stay near noise (c1–c2).
- Global adaptive signals don't work:
  - Main occupancy saturates, so a gate on it falls back to the baseline rule.
  - A ghost-return rate is inflated by the ghost being larger than the small queue, and it fired on the wrong traces.
  - An unweighted ARC-style split drifts toward a larger small queue (c2–c3).
- Extra protection in a full main always evicts someone useful. Flat ghost-return credit (c3) and a protected lap for freq-saturated keys (c7) both lost, the latter by putting entries ahead of valuable quick freq-0 returnees.
- Re-admitting keys to small displaces small residents, which costs more than the main pollution it avoids, especially at the small size. This covers relaps, whether they reset freq (c5) or keep the freq-1 signal (c6, where they duplicated the ghost path).
- Ghost-path routing tuning is exhausted: windows, ceilings on freq ≥ 1 returnees, and stacked reuse rewards (c6–c7).
- New producers of ghost entries that share the ghost FIFO or the stamp clock shrink the age gate's effective window and hurt the small size (c7).
- Per-op-time and metadata-size have never appeared in the fitness vector, for seven cycles running. Several cost-only variants remain unmeasured, and hypotheses repeatedly assumed these dimensions were already being reported.

## Open questions

- What limits web_0 at the large size? It resisted every ghost/small change and moved only marginally when main victims were touched. First-admission filtering of never-seen keys and the small/main split at the large size remain unprobed.
- If main-victim ghosts are decoupled (with their own budget, or without advancing stamps), do c7's large-size gains survive without the small-size loss?
- How much do the cost changes actually save on per-op-time and metadata-size, and does alignment cancel the ghost-deque saving on 8-byte keys?
- Would a gate that directly compares the misses avoided by promoting against the main hits lost to displacement beat the per-key heuristics?

## Per-policy notes

- S3-FIFO baseline (strict `freq > 1` graduation, freq reset): a robust default, and the threshold is not worth changing again.
- Small/main split: static and adaptive variants both land near noise. Untested only at the large size as a lever for web_0.
- Ghost path: the current best is the freq-gated plus age-gated (1× turnover) routing of returnees, with quick freq-0 returns going straight to main. Treat it as near-final, and keep any layered mechanism from erasing its per-key freq or age signal.
- Main eviction: second chance already rewards keys that keep getting hit. Admission boosts add little and hurt quick returnees. Re-qualifying main victims is a small lever and needs to be decoupled from the ghost.
- Cost line: refactors stay behaviour-identical and should be merged onto the current policy parent. Their value is unknown until the cost dimensions are measured.
- Trace character: hm_0 rewards real reuse that gets squeezed out of small, especially at the small size. web_0 punishes late one-off rereferences and keys reused exactly twice. Nearly all gains are small and concentrated at the small cache size.
