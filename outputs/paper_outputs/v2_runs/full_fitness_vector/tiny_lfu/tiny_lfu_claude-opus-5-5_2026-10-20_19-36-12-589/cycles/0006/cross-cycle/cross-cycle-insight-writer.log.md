# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:19:52+00:00

## timeline

### 20:20:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that only affect efficiency or layout and leave behaviour identical are the most reliable gains. Cheap hashing (c1), compact `u32` links (c2), CLOCK ref bits in place of LRU links (c6), and packing flags into link bits (c5) all cut cost or metadata and left hit rate unchanged.\n- Every hit-rate gain so far has come from making the sketch more accurate or the duel's inputs fairer: conservative update (c3), `u16` counters at twice the width (c4), and counting each access once (c5). Each gain was small, and each was smaller than the one before.\n- The fused, register-held sketch increment is the only latency change with a mechanism in the code that recurs. Its gain showed up in c4 and c6. It was absent in c5, which was most likely bench drift.\n- Variants that change one thing at a time on the latest base produce attributions you can read. Merges of orthogonal changes have stayed bit-for-bit identical on hit rate.\n\n## Recurring failures\n\n- Any recency-oriented structure loses on workload-a, which rewards stable frequency filtering at the door:\n  - W-TinyLFU window + SLRU (c1)\n  - a ~1% window (c5)\n  - a 10% window (c6)\n- Sketch aging or halving lowered hit rate every time it was tried (c2 with 4-bit counters, c4 with full-width counters plus conservative update). Decay removes the margin that keeps scans out.\n- Tweaks to the admission rule do nothing or hurt. Letting newcomers win ties (c3) hurt, and frequency-aware victim sampling near the LRU end (c2) did nothing. The strict `>` duel is not over-rejecting.\n- Sketch layouts that break row independence, such as a blocked or cache-line layout (c3), add correlated collisions and inflate estimates. The sketch already fits in cache.\n- Extra work on hot or rare paths costs latency right away: per-op branches, extra hashing on a miss, unbounded CLOCK sweeps, and re-hashing during backward-shift deletion. Tail regressions come from these rare paths.\n- A custom index table that stores only node indices (c6) does worse than SwissTable, because each probe has to load from `nodes`.\n- Per-op timing is noisy across cycles. Several variants sped up or slowed down with no cause in the code. Treat a latency claim as unverified until it has been measured side by side against the parent.\n\n## Open questions\n\n- Can word-level bit scanning (`trailing_ones` over `u64` ref bits) bound CLOCK's sweep and bring `put` p99 back to LRU's level?\n- Does CLOCK combined with the fused increment keep both gains when measured side by side against each parent?\n- If relinking and duplicate hashing don't set the median `get` cost, what does? Map probing, key hashing, or memory layout of the nodes?\n- Would a map that stores hash tags or home slots beat std `HashMap`, or is the map not a bottleneck at all?\n- Is workload-a's hit rate at its ceiling for the TinyLFU family? If it is, the remaining work is metadata and tail-latency reduction only.\n- The unkeyed fast hasher has never been checked against adversarial or collision-prone keys.\n\n## Per-policy notes\n\n- **Baseline TinyLFU (strict-`>` duel against LRU head, no window):** this remains the best-performing policy structure. Its hit rate is set by how accurate the estimates are, not by tie rules or exact recency order.\n- **CountMin sketch:** conservative update plus `u16`/2× width is close to its accuracy limit. Saturating `u16` counters are enough. Further work on width, accuracy or micro-optimisation is used up, apart from the fused increment.\n- **Aging/decay:** this has failed with both counter resolutions and both update rules. Drop it for this workload. At most, a very mild form, such as a much longer period or counting only misses, is untested.\n- **W-TinyLFU / admission window:** this has failed at 1% and at 10%, with and without SLRU. Drop it.\n- **LRU vs CLOCK:** CLOCK keeps hit rate the same and shrinks per-entry metadata. It is the current base, but its eviction sweep has to be bounded.\n- **Victim selection:** sampling near the LRU end gives no information, because those entries' estimates are nearly equal.\n"
}
```

### 20:20:22 chat

I've written `cross-cycle-insights.md` with the four required sections, built only from the ledger for cycles 0001–0006 (c1–c6 below).

Main points across the six cycles:

- **What pays off:** changes that cut cost or metadata without changing behaviour. Those were cheap hashing (c1), compact links (c2) and CLOCK in place of LRU (c6). Every hit-rate gain came from making the sketch more accurate or the admission contest fairer (c3, c4, c5), and each gain was smaller than the last.
- **What keeps failing:** admission windows (tried at about 1% and 10%), periodic sketch aging (tried twice, both times lowered hit rate), changes to the tie rule, sketch layouts that break row independence, and extra work on hot or rare paths.
- **Latency readings:** they have been noisy across cycles. The file says a latency claim only counts once it's measured side by side against its parent.
- **Open questions:**
  - Can CLOCK's eviction sweep be bounded so the `put` tail comes back down?
  - What actually sets the typical `get` cost?
  - Is workload-a's hit rate at its ceiling for this design family?
  - The unkeyed fast hasher has never been checked against adversarial keys.

The per-policy notes treat the plain admission contest with no window as still the best structure, with the sketch's accuracy close to its limit, aging and windows to be dropped, and CLOCK as the current base.


---

- finished: 2026-10-20T20:20:22+00:00
- duration: 30.59s
- status: ok
- input_tokens: 4
- output_tokens: 2084
- cache_read_tokens: 38303
- cache_write_tokens: 11239
- cost_usd: 0.139949
