## Recurring wins

- Changes that stay on one axis combine cleanly. In both cycles the winners touched separate parts of the cache (residency, hashing/map key, sketch storage), and merging winners has produced further gains with no conflicts.
- Cutting hashing and memory-access cost keeps paying off for per-op-time. Moving from SipHash to a single cheap hash, prehashing the map key, and laying the sketch out in cache-line blocks each improved latency. Once sketch misses were cache-line-local, the get-tail regression went away.
- Bit packing works well for metadata and latency. Nibble counters shrank the sketch and made aging cheap. Sentinel-tagged links removed the per-node region tag. Both shrank memory without adding latency.
- The W-TinyLFU window plus SLRU structure has been the most reliable hit-rate gain, and it won in both cycles.
- Aging only pays off once counters are packed. Over packed 4-bit counters it improved hit-rate and latency together.

## Recurring failures

- Aging over wide (u32) counters costs median latency because of the sweep and heap cost. It has been tried once and should not be tried again.
- Runtime adaptivity (hill-climbing window size) adds bookkeeping in `get` and rebalance cost on every op. Its hit-rate gain was limited to the small cache and may come from skipping the frequency contest, not from tuning the window.
- Every hit-rate mechanism tried so far (window/SLRU, aging, adaptive window) gains a lot at the small cache, very little at the large one, and sometimes slips slightly at large capacity. The pattern so far: anything that forgets or bypasses frequency evidence hurts at large sizes.
- Attributions for changes in tail latency (`put` tail improving, `get` tail rising) have been weak or unexplained more than once. Treat them as low-confidence until they reproduce.

## Open questions

- Does merging the window/SLRU structure (with prehashed key) and the blocked 4-bit aging sketch add up the way the previous merge did?
- What drives the large-cache hit-rate slip: 4-bit saturation, a fixed aging sample period, or correlated collisions within a block? Would tuning the sample period or counter width per size fix it?
- Does the adaptive window's small-cache gain survive with the frequency contest restored and a step floor that stops the climbing? This is only worth testing if the merged variant still leaves a small-cache gap.
- Some costs are not measured: the hash stored in each map entry and the growth of the fixed struct. The metadata metric should count them before memory comparisons are trusted.
- `NIL` sentinel encoding has not yet been applied to `next` links.

## Per-policy notes

- **W-TinyLFU (window + SLRU):** The main hit-rate driver, strongest at small capacity. Its region-tag cost was solved by sentinel-tagged links. It pairs well with the fast-hash path and the prehashed map key.
- **Count-min / frequency sketch:** Layout matters more than the counting rule. Cache-line blocking plus 4-bit counters with aging is the current best form. Aging over unpacked counters is a dead end. The large-cache slip is still unexplained.
- **Hashing:** One cheap hash per key, reused for the map key, is the established fast path. Do not go back to computing more than one hash per op.
- **Adaptive window (hill climber):** Negative so far: it raises latency on every op and gains only at small sizes. Shelve it unless a contest-preserving version that stops at its step floor is justified.
- **Baseline admit-or-drop with a never-forgetting sketch:** Superseded. Its losses came mostly from admission contests at small capacity.
