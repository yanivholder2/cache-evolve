# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-11-06T08:21:49+00:00

## timeline

### 08:22:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Two-hit gate + MRU demotion is the retention core (0001–0003).** Gating promotion on a second stash hit raised hit rate everywhere. Demoting normal-tier victims to the stash MRU end added more, with or without the gate. Every later line keeps both.\n- **\"Don't evict a proven resident\" pays for hit rate.** Probing overflow and the probing room gate (0002–0004) gained a little everywhere. Overflow also cut demotion cascades, which helped `put` p99 because stash kept draining.\n- **hm_0 at the large size is the cell that reacts most to retention changes.** It gains most when retention improves and loses most when it drops (0001–0005). Use it as the first check for any retention change.\n- **The stash probe was the `get` p50 gap, and a dense presence mask fixes it (0007).** Skipping the stash scan with a mask test was the first change to lower p50 with identical hit rate.\n- **Cheap hashing is a big lever (0008).** Swapping SipHash for an fmix64-style hasher improved every per-op percentile. It also shrank the mask-refresh cost, because every refresh re-hashes.\n- **Keep the read-side mask word dense; upkeep can change around it (0008–0009).** O(1) clears through a collision word, and a k=2 Bloom test in the same u64, each won with bit-identical hit rate. False-positive rate is now the main tuning axis for the filter.\n- **Reusing existing per-item fields (e.g. `lfu_counter`) keeps metadata size unchanged.** Prefer this for new per-item state.\n\n## Recurring failures\n\n- **Lookup-path tweaks other than the stash probe.** Checking stash last (0003), reversing scan direction (0005), skipping the MRU shift on near-MRU hits (0006) and hashing once per lookup (0007) all left p50 flat or made it worse.\n- **Skipping or deferring cascades moves their cost; it does not remove it (0004–0007).** Unbounded deferral jammed stash and made `put` much worse. A single bounded deferral moved the cost to the `get` tail and to small-cache hit rate.\n- **Deferral signals taken from stash bucket state cannot separate harmful deferrals from harmless ones.** Fill level hardly ever fires because stash is almost always full (0006). The marked-LRU signal only moved cost between `get` and `put` (0007).\n- **Demotion placement based on reuse within one normal-tier stay** (unreused items to stash LRU, 0005) lost hit rate everywhere.\n- **Ungated overflow line (0003–0004):** items placed in probing are found last and get evicted, so the `get` p99 cost outweighed what overflow saved on `put`.\n- **Read-side mask structures other than one dense word** (a shrunken counting mask, or per-slot fingerprints scanned in the bucket, 0008) lost the p50 gain. The first raised false positives; the second was slower than one AND.\n- **Adding work to the stash-hit promotion shift** lands directly on the `get` tail (0008 v002, partly 0006 v003).\n\n## Open questions\n\n- **Combining k=2 with the O(1) clear:** does the fallback rate stay low under k=2 once collisions are tracked per bit? Is k=3 worth it given how full the buckets are?\n- **The `get` p99 is still on the stash-hit promotion path.** How much comes from the item shift and how much from the refresh fallback? Can the shift be made cheaper without adding work to it? The fused rotate looked promising in 0006 but needs to be kept to the third-hit path or to non-inlined code.\n- **Is there p50 left in the segment-level hash, or in remaining mask false positives?**\n- **Stash victim selection:** a two-slot window almost never fired (0009). Does a whole-bucket window with protect-once move hit rate, or is stash eviction a weak lever on these traces?\n- **Does the cheap hasher and mask change the trade-offs on the deferral line?** Revisit only with a new kind of signal.\n- **Noise:** small p99 moves on untouched paths keep appearing (0005–0009), and every result has one sample. Rerun before trusting small tail changes.\n\n## Per-policy notes\n\n- **Dash (only policy in the ledger).** Current base: 0008 v003 (gate, MRU demotion, probing overflow, probing room gate, fast hasher, dense stash mask) plus the 0009 mask winners (O(1) collision-word clear; k=2 Bloom test).\n- **Dash hit rate has plateaued since about 0004.** Recent wins are cost-only. Hit-rate ideas still open: stash victim choice and a better deferral signal.\n- **Dash cost model:** hashing and the stash probe drive `get` p50. The stash-hit promotion shift drives `get` p99. Stash churn (removals, mask upkeep, cascades) drives `put`.\n- **Small vs large cache in Dash:** longer stash residency helps the large cache (proven residents stay in place) but hurts the small cache (stash slots are scarce).\n"
}
```

### 08:22:15 thinking



### 08:22:15 chat

I've rebuilt `cross-cycle-insights.md` in the cycle 0009 cross-cycle output directory from the ledger for cycles 0001–0009. It uses the four required sections in order and gives no numbers.

What it says, in short:

- **What keeps working:** the two-hit gate plus MRU demotion is still the source of the hit-rate gains. A dense one-word stash mask lowered `get` p50, and swapping in a cheap hasher improved every per-op percentile. In 0009, a faster way to clear mask bits and a two-bit-per-key mask both won without changing hit rate.
- **What keeps failing:** lookup tweaks other than the stash probe left p50 flat or made it worse. Skipping or deferring cascades only moved their cost to another metric. Deferral signals read from stash bucket state didn't work, and neither did mask designs other than one dense word. Adding work to the promotion shift hurt the `get` tail.
- **Open threads:**
  - Does the fast clear still pay off once the two-bit mask is in place?
  - Is the remaining `get` p99 cost in the promotion shift or in the mask-refresh fallback?
  - Is stash victim choice worth any hit rate on these traces?
  - Small tail changes need reruns, since every result so far has a single sample.
- **Per-policy notes:** Dash is the only policy in the ledger. The notes give its current base, record that hit rate has stalled since about cycle 0004 so recent wins are speed-only, and list which costs drive which metrics.


---

- finished: 2026-11-06T08:22:16+00:00
- duration: 26.48s
- status: ok
- input_tokens: 4
- output_tokens: 2356
- cache_read_tokens: 42575
- cache_write_tokens: 14454
- cost_usd: 0.172256
