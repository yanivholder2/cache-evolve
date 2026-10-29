# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-29T10:52:28+00:00

## timeline

### 10:52:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan-resistant insertion (cycle 1) was the one big hit-rate gain: new one-touch items get evicted first, through LIP quarter-depth insertion or a cleared CLOCK bit. Gains are largest on wide (1024) buckets. Every policy change since then has been marginal.\n- Wins on the get/put miss path come from cutting the number of full-key compares, not from scan order. The dense 1-byte fingerprint pre-filter (cycle 5) helped get and put at both p50 and p99.\n- On put, the best results came from replacing generic memmoves with one hand-written `ptr::read`/`ptr::copy`/`ptr::write` partial rotation. The fused evict-plus-insert (cycle 6) was confirmed on p50 and p99.\n- Shrinking `Item` (24 bytes, cycle 4) is the only lever that has moved metadata-size. Once the global atomic tick was gone, it was also at least speed-neutral.\n- Mechanical rewrites that keep the layout reproduce every hit-rate cell exactly, every time. The current policy (LIP quarter-depth plus BIP) is a stable base for speed work.\n- A static policy split by bucket width (BIP 1/32 narrow, 1/128 wide, cycle 7) did what it predicted: web_0 wide improved and narrow cells stayed bit-identical.\n\n## Recurring failures\n\n- Reuse and adaptation signals have now failed the hm_0 vs web_0 tradeoff five times. The attempts: BIP's blind period, a 16-entry ghost ring, a hashed ghost bit set, adaptive depth driven by hit position, and a 1-bit second chance with no decay. The causes repeat: the window doesn't match the reuse distance, the filter saturates or fills up, or the signal mixes new admits with items that have aged down.\n- Extra work on the put-miss path keeps cancelling the gain it sits next to. Examples: per-op string compares, dynamic dispatch, SipHash, full-window sweeps, and side structures.\n- Any reordering or hit-time write beyond a single store loses CLOCK's latency advantage. Self-organizing swaps and changes to scan order did not move get p50.\n- Side arrays only pay off when very dense. A `u64` key mirror in a separate Vec hurt locality, while byte tags worked.\n- Generic standard-library helpers can hide slow scalar paths: `rotate_left(1)` on short suffixes lost to plain memmoves.\n- Removing the atomic clock is speed-neutral on the tag/fused base, even with stride pinned. Do not expect a speed gain from it.\n- Single-run per-op-time, especially get-p99, has drifted in cycles 4, 6 and 7, moving on code paths that were never touched. It is not a valid judge without same-session repeated runs against the parent.\n- Metadata-size does not see per-bucket state (key mirror, ghost ring, tags, bucket fields). A flat reading there says nothing about real footprint.\n\n## Open questions\n\n- Does an inline, devirtualised ClassicLRU hit path (MRU-skip, manual `ptr::copy` shift for items and tags) finally lower get p50?\n- What is the remaining put cost after the fused rotation (the quarter-length shift plus the tag mirror)? Could a ring offset over the probationary region or tags-only movement cut it?\n- On wide buckets, is a BIP period between 1/32 and 1/128 (such as 1/64) better than either end, or is the web_0 vs hm_0 tradeoff flat? Allow one probe at most.\n- Does bucket stride or layout growth (extra fields) add hidden cost? Check by merging fields into one shared `u32`.\n- Can tags be extended to CLOCK, or widened or SIMD-masked, to cut the remaining false-positive compares?\n- Is a reuse bit worth one last try? Only if it has independent decay (sweep, or cleared at quarter depth) and places spared items outside the admission region.\n- Do the apparent regressions from cycles 6 and 7 (clock removal, width-keyed BIP) go away in a back-to-back same-session rerun?\n\n## Per-policy notes\n\n- ClassicLRU / LIP: quarter-depth insertion is the best hit-rate base. Fixed depth beats adaptive depth. The cost of insert-at-depth is now handled by the fused partial rotation. Make policy checks once at construction, never per op.\n- BIP (on LIP): a fixed 1/32 MRU admission helps small-bucket hm_0 and costs some web_0. Keying the period to bucket width (1/128 wide) roughly trades the web_0 gain against hm_0 on wide buckets. Ghost or filter-based replacements for the period have all done worse.\n- CLOCK / GCLOCK: the one-byte hit store is its main speed asset, so keep hit-path writes to that store. A 2-bit GCLOCK with a bounded 16-slot sweep and clear-on-insert gave small, consistent hit-rate gains and a better put tail. No tag pre-filter has been tried here yet.\n- Frequency-aware victim window (cycle 1): the result was identical to the baseline, probably because the harness never ran that path. It was never properly evaluated.\n- Second chance at the LRU end: the bit saturates on large caches without independent decay. It helped only small hm_0, and its rotations hurt the put tail.\n"
}
```

### 10:52:54 chat

I wrote `cross-cycle-insights.md` in the cycle 0007 cross-cycle folder, built fresh from the seven-cycle ledger and using the four required sections.

What the file says:

- **Wins that held up:** the clearest are the quarter-depth insertion that pushes new items out first (cycle 1), the 1-byte fingerprint filter on lookups (cycle 5), and the combined evict-and-insert step on put (cycle 6). The smaller `Item` (cycle 4) is the only change that has ever moved metadata-size.
- **Main failure:** five attempts to detect key reuse have failed to help both workloads at once. They either saturate, fill up, or watch the wrong time window.
- **Measurement warning:** single-run timing numbers have drifted in three cycles on code nobody touched. Only repeated runs against the parent in the same session should count. The metadata-size measure also can't see extra state stored per bucket.
- **Open questions:** the main ones are a faster hit path built with hand-written copies, the remaining cost of the shift on put, and one more test of the wide-bucket BIP rate. The full list is in the file.
- **Per-policy notes:** separate notes for ClassicLRU/LIP, BIP, CLOCK/GCLOCK, the frequency-aware victim window (never actually tested, since the harness likely never ran it), and second chance.


---

- finished: 2026-10-29T10:52:55+00:00
- duration: 26.43s
- status: ok
- input_tokens: 4
- output_tokens: 2411
- cache_read_tokens: 41557
- cache_write_tokens: 13581
- cost_usd: 0.165195
