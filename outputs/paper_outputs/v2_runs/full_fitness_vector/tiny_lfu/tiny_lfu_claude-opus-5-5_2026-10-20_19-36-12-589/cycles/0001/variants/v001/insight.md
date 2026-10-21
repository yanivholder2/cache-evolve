# Variant v001 Insight

## Helped

- hit-rate: Only a marginal, near-noise improvement. The ~1% window (about 20 slots at capacity 2048) just delays the same sketch-frequency contest the baseline already ran. SLRU protection of reused keys barely changes which keys survive on workload-a.

## Hurt

- metadata-size: `Node` gains a `region: u8` tag, and alignment pads it out to a full extra word. The cache struct also grows, because the single `head`/`tail` pair becomes per-region `heads`/`tails`/`lens` arrays plus the `window_cap`/`protected_cap` fields.
- per-op-time: Every hit now does a full `unlink` + `push_back` through region-indexed arrays. A probation hit also promotes the entry to protected and demotes the protected LRU, which is two relinks and more scattered arena writes. That shows up most in get tail latency. Full-cache puts now juggle a window candidate plus a probation/protected victim and clone the evicted key for `map.remove`, so put latency also rises.

## Takeaway

Moving from TinyLFU to W-TinyLFU (window + SLRU), with the `CountMinSketch` left unchanged, made the cache costlier in both memory and latency without buying meaningful hit rate on this trace. The admission decision still depends entirely on a sketch that has no aging, so changing the list structure alone did little. Future hit-rate work should change the frequency signal, for example by adding counter decay/reset or resizing the window. A further restructuring of the resident lists is unlikely to help, and if this structure is kept, its per-hit relinking cost needs to come down.
