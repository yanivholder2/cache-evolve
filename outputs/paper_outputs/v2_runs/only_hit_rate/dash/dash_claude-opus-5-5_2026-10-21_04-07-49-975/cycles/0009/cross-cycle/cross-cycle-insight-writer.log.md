# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T05:10:28+00:00

## timeline

### 05:10:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Filtering beats protecting.** From 0001 to 0009, the winners made it harder to get into scarce or protected space: a stricter stash→target admission bar (0001), rarer and then no blind BIP MRU admission (0006, 0008), and a recency gate on ghost hits (0007). Variants that added protection, or let weaker evidence in, lost or broke even.\n- **Fresh keys face a harsh bar, proven ex-residents are protected (an SLRU/2Q-like split).** This first appeared as MRU stash re-entry for demoted items (0002). It was then applied through where items are placed: fresh keys land at the LRU end (0004), and only evidence-backed keys go to MRU (0005–0008).\n- **Where a key is placed is the lever, not its counter.** Pre-credit that only changes a counter did nothing (0004). The same ghost history worked once a ghost hit changed where the key landed (0005).\n- **Recency matters more than amount of evidence.** A bigger ghost window lost (0006). A gate at about half a window won (0007). A ghost table halved to fit the gated horizon held hit-rate (0009). Useful ghost history ends somewhere under one cache's worth of evictions.\n- **Simplification often wins outright.** Removing BIP (0008) and shrinking the ghost (0009) won while cutting state and branches. Removing a redundant path is worth trying before adding a mechanism.\n- **Orthogonal filters stack.** Gating the selective path and removing the blind path compounded (0008). They stacked because they act on different admission routes.\n\n## Recurring failures\n\n- **Stacked retention on the same items.** Scan-skip for credited or ghost-admitted keys (0003, 0006) and a fresh-key doorkeeper (0009) added nothing. MRU placement plus the second-chance scan already protect those items.\n- **Taking protected capacity for fresh keys.** Wider probation (0005), set-dueling DIP (0005), MRU landing in the normal tier (0001) and an oversized ghost window (0006) all lost. On this trace the protected stash capacity is close to zero-sum.\n- **Counter or frequency gating where position already filters.** A frequency bar on target→probing (0002), a stash threshold of 3 or more on a mature base (0003), and a TinyLFU duel against the LRU-end resident (0004) were all flat. Position and frequency are coupled in this design.\n- **Victim choice and routing tweaks.** Cascade-free routing (0003, 0007, the second time bit-identical to its parent), stash victim choice (0003) and spill-to-free-slot (0001) were all flat. Any victim they pick is cheap to recover, and free slots are rare in steady state.\n- **Fine-tuning that has plateaued.** Gate-cutoff sweeps, graded mid-stash placement (0008) and shifting the stash/normal slot ratio (0009) moved hit-rate only at noise level. Capacity looks roughly interchangeable between the tiers.\n- **Process failure: unmeasured axes.** Efficiency hypotheses (per-op-time, metadata) keep being proposed and keep \"winning\" on dimensions nobody measured. Writers have never inspected the overlay source, so a no-op cannot be told apart from a real null result.\n\n## Open questions\n\n- Do any of these results hold on a second trace or cache size? Every result so far comes from one LIRS trace at one cache size. The biggest risk is that removing BIP and the short ghost horizon overfit that trace, because they would lose on traces whose useful reuse falls outside the horizon.\n- Metadata-size and per-op-time have never been reported. Are the claimed savings real for the halved ghost, BIP removal and cascade-free routing?\n- How small can the ghost get before collisions cost hits (a quarter, an eighth)? Would a set-associative ghost push that point further out?\n- Does the cascade/displacement path ever fire at this cache size? Did the bit-identical variants (0003 v001, 0007 v003) actually apply their changes?\n- Hit-rate levers inside the stash look exhausted. What is left must change who reaches the stash at all, or must come from structural changes measured on new workloads.\n\n## Per-policy notes\n\n- **Current lineage (stash plus normal tier, ghost-guided):** the base is now fresh keys at stash LRU, a second-chance scan, MRU re-entry for demoted ex-residents, MRU admission on recency-gated ghost hits, no BIP, and a ghost about half the cache in size. Hit-rate on this trace has plateaued since about 0008, and further gains are at noise level.\n- **Normal tier:** buckets are almost always full, and Front landing already filters the target→probing edge. The neighbour demotion cascade stopped mattering once the stash could recover victims cheaply.\n- **Stash:** this is where the real gains have come from. Its value comes from being an evidence-gated victim cache, not from its size or its victim-selection rule.\n- **BIP/DIP:** these helped only before the ghost existed. Afterwards they were a net cost, and the best setting is off. Adaptive dueling was noisy and lost.\n- **Ghost table:** it helps only when a ghost hit changes placement and the hit is recent. Its size should match the recency gate, not exceed one cache.\n"
}
```

### 05:10:54 chat

I've written `cross-cycle-insights.md` in the cycle 0009 cross-cycle folder, rebuilt from the cycle 0001–0009 ledger with the four required sections in order.

The main conclusions:
- **What keeps winning:** making it harder to get into protected stash space works better than adding protection. That held from cycle 0001 to 0009. Changing where a key lands helps; changing its counter doesn't. A recent ghost hit counts for more than a long ghost history.
- **What keeps failing:** stacking a second protection on items that are already protected, giving protected space to fresh keys, frequency gating where placement already filters, and tweaking victim choice or routing. All came out flat or worse.
- **Hit-rate has stalled:** gains on this trace have been at noise level since about cycle 0008. The remaining levers inside the stash look used up.
- **Biggest risk:** every result comes from one LIRS trace at one cache size. Removing BIP and keeping only a short ghost horizon may overfit that trace.
- **Unmeasured claims:** per-op-time and metadata-size have never been reported, so the efficiency "wins" (including 0009's halved ghost table) are unconfirmed. Nobody could inspect the overlay source either. So for the variants that matched their parent exactly, there's no way to tell whether the change had no effect or never reached the build.


---

- finished: 2026-10-21T05:10:55+00:00
- duration: 27.47s
- status: ok
- input_tokens: 4
- output_tokens: 2358
- cache_read_tokens: 41360
- cache_write_tokens: 14598
- cost_usd: 0.173218
