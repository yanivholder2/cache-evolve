# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-29T08:28:44+00:00

## timeline

### 08:28:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-29_08-15-12-815/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that stay on one axis combine cleanly. In both cycles the winners touched separate parts of the cache (residency, hashing/map key, sketch storage), and merging winners has produced further gains with no conflicts.\n- Cutting hashing and memory-access cost keeps paying off for per-op-time. Moving from SipHash to a single cheap hash, prehashing the map key, and laying the sketch out in cache-line blocks each improved latency. Once sketch misses were cache-line-local, the get-tail regression went away.\n- Bit packing works well for metadata and latency. Nibble counters shrank the sketch and made aging cheap. Sentinel-tagged links removed the per-node region tag. Both shrank memory without adding latency.\n- The W-TinyLFU window plus SLRU structure has been the most reliable hit-rate gain, and it won in both cycles.\n- Aging only pays off once counters are packed. Over packed 4-bit counters it improved hit-rate and latency together.\n\n## Recurring failures\n\n- Aging over wide (u32) counters costs median latency because of the sweep and heap cost. It has been tried once and should not be tried again.\n- Runtime adaptivity (hill-climbing window size) adds bookkeeping in `get` and rebalance cost on every op. Its hit-rate gain was limited to the small cache and may come from skipping the frequency contest, not from tuning the window.\n- Every hit-rate mechanism tried so far (window/SLRU, aging, adaptive window) gains a lot at the small cache, very little at the large one, and sometimes slips slightly at large capacity. The pattern so far: anything that forgets or bypasses frequency evidence hurts at large sizes.\n- Attributions for changes in tail latency (`put` tail improving, `get` tail rising) have been weak or unexplained more than once. Treat them as low-confidence until they reproduce.\n\n## Open questions\n\n- Does merging the window/SLRU structure (with prehashed key) and the blocked 4-bit aging sketch add up the way the previous merge did?\n- What drives the large-cache hit-rate slip: 4-bit saturation, a fixed aging sample period, or correlated collisions within a block? Would tuning the sample period or counter width per size fix it?\n- Does the adaptive window's small-cache gain survive with the frequency contest restored and a step floor that stops the climbing? This is only worth testing if the merged variant still leaves a small-cache gap.\n- Some costs are not measured: the hash stored in each map entry and the growth of the fixed struct. The metadata metric should count them before memory comparisons are trusted.\n- `NIL` sentinel encoding has not yet been applied to `next` links.\n\n## Per-policy notes\n\n- **W-TinyLFU (window + SLRU):** The main hit-rate driver, strongest at small capacity. Its region-tag cost was solved by sentinel-tagged links. It pairs well with the fast-hash path and the prehashed map key.\n- **Count-min / frequency sketch:** Layout matters more than the counting rule. Cache-line blocking plus 4-bit counters with aging is the current best form. Aging over unpacked counters is a dead end. The large-cache slip is still unexplained.\n- **Hashing:** One cheap hash per key, reused for the map key, is the established fast path. Do not go back to computing more than one hash per op.\n- **Adaptive window (hill climber):** Negative so far: it raises latency on every op and gains only at small sizes. Shelve it unless a contest-preserving version that stops at its step floor is justified.\n- **Baseline admit-or-drop with a never-forgetting sketch:** Superseded. Its losses came mostly from admission contests at small capacity.\n"
}
```

### 08:29:05 chat

I've written `cross-cycle-insights.md` from the cycle 0001 and 0002 ledger entries, using the four required sections in order.

- **Recurring wins:** Changes that each touch one part of the cache combine cleanly when merged. Cutting hashing and memory-access cost keeps improving latency. Bit packing shrank the metadata without adding latency. The W-TinyLFU window plus SLRU won in both cycles. Aging only helps once its counters are packed into 4 bits.
- **Recurring failures:** Aging over u32 counters costs median latency. The hill-climbing window adds latency to every operation and only gains at the small cache size. Every hit-rate change so far gains mostly at the small size and sometimes slips slightly at the large one. Explanations for changes in tail latency have been weak more than once.
- **Open questions:** Whether merging the cycle 0002 winners adds up the way the last merge did. What causes the large-cache hit-rate slip. Whether the hill climber's small-cache gain survives once the frequency contest is restored. Two costs the metadata metric doesn't count yet: the hash stored in each map entry and the growth of the cache struct. Whether to apply the `NIL` sentinel to `next` links.
- **Per-policy notes:** Grouped under the window plus SLRU, the frequency sketch, hashing, the adaptive window, and the original baseline.

As the constraints require, the file uses no numbers and doesn't draw on the previous version.


---

- finished: 2026-10-29T08:29:05+00:00
- duration: 21.89s
- status: ok
- input_tokens: 4
- output_tokens: 1869
- cache_read_tokens: 36078
- cache_write_tokens: 7528
- cost_usd: 0.104836
