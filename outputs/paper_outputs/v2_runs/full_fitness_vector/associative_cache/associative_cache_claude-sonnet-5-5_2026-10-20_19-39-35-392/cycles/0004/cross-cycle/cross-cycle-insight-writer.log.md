# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:55:02+00:00

## timeline

### 19:55:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant insertion/eviction policy is the only lever that has moved hit rate (cycle 0002 onward). One-hit-wonders must not displace reused entries.\n- Mid-bucket (about len/4 to mid) insertion plus explicit transpose-by-one promotion on hit is the best policy shape found so far.\n- Cheap, stateless mechanisms beat stateful ones. Insertion position and a reference bit beat frequency counters.\n- Put latency improves when per-put work shrinks (O(1) swap insertion, hash-once indexing). Where the new entry lands matters; the exact order of the rest does not.\n- No winning variant has needed extra metadata.\n\n## Recurring failures\n\n- Geometry sweeps (bucket width) and position-only LRU tweaks: hit rate is insensitive, so conflict misses are not the bottleneck (cycle 0001).\n- Rotate/memmove and arithmetic/hash micro-optimizations: no get gain in four cycles. Item movement and arithmetic do not dominate get time.\n- Stacking scan-resistance mechanisms (reference bit plus low insertion) does not compound and can lose both hit rate and speed.\n- Pushing insertion too close to the LRU end starves new entries of the chance to prove reuse.\n- Weak admission signals (one-entry ghost, per-slot frequency counter) add bookkeeping cost for little or negative hit-rate return.\n- Extra per-access logic (branches, masking, shifts) consistently costs latency.\n- Isolated latency shifts with no mechanism (put gains in untouched paths, small get shifts) have repeatedly been noise or artifacts. Discount them.\n- Winner status has been awarded on thin or negative evidence (e.g. variants whose own insight reports a small loss). Treat those results as unconfirmed.\n\n## Open questions\n\n- Where does median get time go? Memory layout (cache-line alignment, tag/key packing, prefetch) is the untested suspect, and this has gone unmeasured for four cycles.\n- Do any gains hold on a workload other than workload-a, or are they specific to its scan-heavy traffic?\n- Is scan resistance already saturated by insertion position, or would a deeper or more selective admission history help?\n- Can hash-once indexing be made to help get rather than only put?\n- Which Winners are real and which are within noise?\n\n## Per-policy notes\n\n- Baseline LRU: the position and geometry axes are exhausted. Partial promotion hurts hit rate.\n- CLOCK reference bit: cheap and cut reordering cost alone, but a one-bit second chance is too coarse when stacked on low insertion.\n- Frequency counter (2-bit, LRU tie-break): small hit-rate gain, slower get and put. Dropped.\n- Low-position insertion (LIP/BIP-style): clearest single-mechanism hit gain, but at the extreme LRU end it starves new entries.\n- Mid-insertion plus transpose promotion: current policy baseline. The O(1) swap form removes the shift cost and is the preferred implementation.\n- Ghost admission filter (one entry per bucket): too weak, hurts reusable keys. Dropped.\n"
}
```

### 19:55:18 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle output directory. It has the four fixed sections in order: Recurring wins, Recurring failures, Open questions and Per-policy notes. I built it only from the four-cycle ledger digest.

- **Recurring wins:**
  - Scan-resistant insertion is the only lever that has moved hit rate.
  - Mid-bucket insertion with transpose promotion is the best policy shape so far.
  - Cheap, stateless mechanisms beat stateful ones.
  - Put latency improves when per-put work shrinks.
- **Recurring failures:**
  - Geometry sweeps and position-only LRU tweaks.
  - Rotate, arithmetic and hash micro-optimizations, which have given no get gain in four cycles.
  - Stacking scan-resistance mechanisms.
  - Weak admission signals.
  - Latency shifts with no mechanism behind them, which I flagged as noise.
  - Winners awarded on thin or negative evidence, which I flagged as unconfirmed.
- **Open questions:** where median get time goes, with memory layout the main untested suspect, and whether the gains hold on a workload other than workload-a. I also listed whether scan resistance is already saturated, whether hash-once indexing can help get, and which Winners are real.
- **Per-policy notes:** one bullet each for baseline LRU, CLOCK reference bit, frequency counter, low-position insertion, mid-insertion with transpose, and the ghost admission filter.

I used no numbers, only qualitative terms.


---

- finished: 2026-10-20T19:55:19+00:00
- duration: 16.72s
- status: ok
- input_tokens: 4
- output_tokens: 1738
- cache_read_tokens: 36611
- cache_write_tokens: 8361
- cost_usd: 0.054924
