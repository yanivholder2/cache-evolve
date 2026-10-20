# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:12:52+00:00

## timeline

### 12:13:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Lean per-op mechanics keep paying off independently of policy: hash once per op, contiguous per-bucket storage, in-place policy updates. Every winner in both cycles has built on them.\n- Merged buckets with one shared eviction policy are the base that works. Each key may use the whole bucket, which recovers hit rate and removes partner-path tail latency.\n- Taking work out of the put path pays off. Contiguous storage first, then removing wall-clock reads and shrinking `Item`, each gave clear put and tail-latency gains.\n- Shrinking metadata (logical ticks instead of `Instant`) was the first change to move metadata-size, at no hit-rate cost.\n\n## Recurring failures\n\n- Placing a key outside its single hashed bucket (two-choice, cuckoo, cross-bucket victim areas) always fails P2 slot_legality. Do not propose it again.\n- Split home/partner halves inside a bucket lowered hit rate in both cycles. The bookkeeping cancels the flexibility, and hot halves push out their partner's items.\n- Get p50 keeps getting worse as policy and layout change. Full-bucket scans and memmoves of large items on hit and evict push cost onto the median read. No cycle has fixed this yet.\n- Explaining latency by guesswork has been wrong before. Removing the per-hit `Instant::now()` was expected to fix get p50 and did not. Attribute regressions only with care.\n\n## Open questions\n\n- Plain LRU vs SLRU on the same combined base (merged bucket + 24-byte `Item`): which wins on hit rate? They have never been measured side by side.\n- Is SLRU's 3/4 protected fraction close to optimal?\n- Can a compact tag/fingerprint array, or a ring/offset layout that avoids front-of-Vec shifts, bring back the get median without losing hit rate?\n- Can the timestamp field be dropped entirely under position-ordered policies to shrink metadata further?\n- Would frequency-aware or admission-filtered policies inside the single-bucket rule add hit rate beyond SLRU?\n\n## Per-policy notes\n\n- Split-half LRU (home/partner): passes gates only when pairs are merged into one physical bucket. Hit rate stays below baseline. Retired.\n- Plain LRU over merged bucket: hit rate above baseline. Get p50 cost comes from scans and shifts. Does not need the timestamp.\n- SLRU (probation/protected): the clearest hit-rate gain so far, which suggests the trace rewards scan resistance. Segment membership is stored in the existing boundary, so it adds no metadata. Same get-median cost as plain LRU. Does not need the timestamp.\n- Timestamp-based policies: wall-clock reads on the hot path hurt put and tail latency. Use logical ticks if any policy needs recency stamps.\n"
}
```

### 12:13:10 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output folder, rebuilt from scratch from the cycle 0001 and 0002 ledger entries. It keeps the four required sections in order.

- **Recurring wins:** three things helped in both cycles:
  - Lean per-operation mechanics: one hash per operation, each bucket stored in one block, policy updated in place.
  - Merged buckets with one shared eviction policy.
  - Shrinking `Item` metadata by using logical ticks instead of wall-clock timestamps.
- **Recurring failures:**
  - Any design that lets a key sit outside its hashed bucket always fails the P2 slot_legality check.
  - Split home/partner halves inside a bucket lowered hit rate.
  - The median read time (get p50) has kept getting worse in both cycles, and nothing has fixed it yet.
  - Cycle 0001 guessed the per-hit clock read was slowing reads. Cycle 0002 showed it wasn't, so latency causes need careful attribution.
- **Open questions:**
  - Plain LRU and SLRU have never been measured against each other on the same base.
  - SLRU's 3/4 protected fraction may not be optimal.
  - A compact tag array or a ring layout might bring the median read time back down.
  - The timestamp field might be dropped entirely.
  - Frequency-aware or admission-based policies might add more hit rate.
- **Per-policy notes:**
  - Split-half LRU is retired.
  - Plain LRU and SLRU both beat baseline on hit rate, with SLRU showing the clearest gain at no extra metadata cost. Both pay the same median-read cost.
  - Any policy that needs recency stamps should use logical ticks, not wall-clock reads.


---

- finished: 2026-10-19T12:13:10+00:00
- duration: 18.25s
- status: ok
- input_tokens: 4
- output_tokens: 1687
- cache_read_tokens: 34738
- cache_write_tokens: 7160
- cost_usd: 0.097984
