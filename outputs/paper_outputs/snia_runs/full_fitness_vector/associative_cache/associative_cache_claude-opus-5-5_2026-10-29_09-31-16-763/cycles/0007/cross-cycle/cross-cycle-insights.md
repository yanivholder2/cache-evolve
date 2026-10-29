## Recurring wins

- Scan-resistant insertion (cycle 1) was the one big hit-rate gain: new one-touch items get evicted first, through LIP quarter-depth insertion or a cleared CLOCK bit. Gains are largest on wide (1024) buckets. Every policy change since then has been marginal.
- Wins on the get/put miss path come from cutting the number of full-key compares, not from scan order. The dense 1-byte fingerprint pre-filter (cycle 5) helped get and put at both p50 and p99.
- On put, the best results came from replacing generic memmoves with one hand-written `ptr::read`/`ptr::copy`/`ptr::write` partial rotation. The fused evict-plus-insert (cycle 6) was confirmed on p50 and p99.
- Shrinking `Item` (24 bytes, cycle 4) is the only lever that has moved metadata-size. Once the global atomic tick was gone, it was also at least speed-neutral.
- Mechanical rewrites that keep the layout reproduce every hit-rate cell exactly, every time. The current policy (LIP quarter-depth plus BIP) is a stable base for speed work.
- A static policy split by bucket width (BIP 1/32 narrow, 1/128 wide, cycle 7) did what it predicted: web_0 wide improved and narrow cells stayed bit-identical.

## Recurring failures

- Reuse and adaptation signals have now failed the hm_0 vs web_0 tradeoff five times. The attempts: BIP's blind period, a 16-entry ghost ring, a hashed ghost bit set, adaptive depth driven by hit position, and a 1-bit second chance with no decay. The causes repeat: the window doesn't match the reuse distance, the filter saturates or fills up, or the signal mixes new admits with items that have aged down.
- Extra work on the put-miss path keeps cancelling the gain it sits next to. Examples: per-op string compares, dynamic dispatch, SipHash, full-window sweeps, and side structures.
- Any reordering or hit-time write beyond a single store loses CLOCK's latency advantage. Self-organizing swaps and changes to scan order did not move get p50.
- Side arrays only pay off when very dense. A `u64` key mirror in a separate Vec hurt locality, while byte tags worked.
- Generic standard-library helpers can hide slow scalar paths: `rotate_left(1)` on short suffixes lost to plain memmoves.
- Removing the atomic clock is speed-neutral on the tag/fused base, even with stride pinned. Do not expect a speed gain from it.
- Single-run per-op-time, especially get-p99, has drifted in cycles 4, 6 and 7, moving on code paths that were never touched. It is not a valid judge without same-session repeated runs against the parent.
- Metadata-size does not see per-bucket state (key mirror, ghost ring, tags, bucket fields). A flat reading there says nothing about real footprint.

## Open questions

- Does an inline, devirtualised ClassicLRU hit path (MRU-skip, manual `ptr::copy` shift for items and tags) finally lower get p50?
- What is the remaining put cost after the fused rotation (the quarter-length shift plus the tag mirror)? Could a ring offset over the probationary region or tags-only movement cut it?
- On wide buckets, is a BIP period between 1/32 and 1/128 (such as 1/64) better than either end, or is the web_0 vs hm_0 tradeoff flat? Allow one probe at most.
- Does bucket stride or layout growth (extra fields) add hidden cost? Check by merging fields into one shared `u32`.
- Can tags be extended to CLOCK, or widened or SIMD-masked, to cut the remaining false-positive compares?
- Is a reuse bit worth one last try? Only if it has independent decay (sweep, or cleared at quarter depth) and places spared items outside the admission region.
- Do the apparent regressions from cycles 6 and 7 (clock removal, width-keyed BIP) go away in a back-to-back same-session rerun?

## Per-policy notes

- ClassicLRU / LIP: quarter-depth insertion is the best hit-rate base. Fixed depth beats adaptive depth. The cost of insert-at-depth is now handled by the fused partial rotation. Make policy checks once at construction, never per op.
- BIP (on LIP): a fixed 1/32 MRU admission helps small-bucket hm_0 and costs some web_0. Keying the period to bucket width (1/128 wide) roughly trades the web_0 gain against hm_0 on wide buckets. Ghost or filter-based replacements for the period have all done worse.
- CLOCK / GCLOCK: the one-byte hit store is its main speed asset, so keep hit-path writes to that store. A 2-bit GCLOCK with a bounded 16-slot sweep and clear-on-insert gave small, consistent hit-rate gains and a better put tail. No tag pre-filter has been tried here yet.
- Frequency-aware victim window (cycle 1): the result was identical to the baseline, probably because the harness never ran that path. It was never properly evaluated.
- Second chance at the LRU end: the bit saturates on large caches without independent decay. It helped only small hm_0, and its rotations hurt the put tail.
