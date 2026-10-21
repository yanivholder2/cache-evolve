## Recurring wins

- Hit-rate and speed/size are independent axes. Hit-rate comes only from policy. Speed and size come only from mechanics (u32 links, lighter hasher, head-touch, key move). This held early on the plain LRU.
- Segmented LRU's scan resistance is established (cycle 1). Ghost-gated cold-key insertion at the LRU end of probation was the one later policy gain (cycles 7-8).
- Trimming ghost overhead (smaller table, fewer hashes, ghost lookup only on a full cache) cut put tail latency without losing hit-rate (cycle 8).
- Removing the hash avalanche fold on the segmented policy sped up put and shrank the struct (cycle 4). Mixing was a real cost there.
- Bounding the second-chance eviction walk with a cap cut put p99. It traded away some hit-rate (cycle 5).
- Larger protected share gave small, diminishing hit-rate gains (cycles 5-8).

## Recurring failures

- Work-shifting does not remove cost. Lazy demotion, a reference bit instead of relink-on-get, and the eviction walk each moved promotion/demotion work between operations (cycles 3-4).
- Ghost-hit promotion fails: sending ghost hits to protected, or to probation MRU, lowered hit-rate (cycles 7, 9). The ghost filter works by making suspicious keys cheap to evict.
- Adding bookkeeping to the put path (ghost counters, admission gates, adaptive caps, reference bits) raised put cost and state with no verified gain (cycle 10).
- Micro-optimizations are invisible: key-clone removal, node-slot reuse, map-call fusion, further ghost shrinking, head-adjacent relink skips (cycles 4, 7, 9, 10). Map calls and allocation are not the put bottleneck.
- Constant and position tweaks are exhausted: protected-share steps, demotion position, `put`-update promotion.
- Winner status is unreliable for small changes. Per-op-time deltas appear on untouched paths in every cycle since 5, so they are noise. Many recent Winners likely reflect noise tolerance, not merit.
- On a segmented policy, u32 links and bit packing gave size savings but no speed (cycle 3).

## Open questions

- What is the per-op-time noise floor? This has been pending since cycle 5. A repeat-run measure is still missing.
- Does the ghost-gated design hold on non-scan workloads? This check has been pending for many cycles.
- Where does put time actually go (hashing, list-pointer cache misses, eviction walk)? A profile is needed before choosing a lever.
- How much heap-allocated ghost memory is there? The per-node size metric cannot see it.
- Is there hit-rate headroom left on this trace, or is the baseline near its ceiling? Is there a lever outside insertion position and segment share?
- Does segment bit packing or implicit segment storage help once the other pieces are in place?

## Per-policy notes

- Plain LRU: u32 links, a lighter hasher and a key move were pure speed and size wins (cycle 2). It has no hit-rate lever.
- Segmented LRU: it gains hit-rate on scans but pays for promotion/demotion relinking on the hot path. Gains here are mostly size or hit-rate, rarely speed.
- Segmented LRU with ghost-gated insertion (the cycle-8 baseline): small ghost table, single-hash gated eviction path, larger protected share. It is the current parent and is near saturation on this trace.
- Second-chance (CLOCK-style) variants: they give a small scan-resistance gain, but get-side savings return as put and eviction cost. A cap makes it a latency/hit-rate dial.
