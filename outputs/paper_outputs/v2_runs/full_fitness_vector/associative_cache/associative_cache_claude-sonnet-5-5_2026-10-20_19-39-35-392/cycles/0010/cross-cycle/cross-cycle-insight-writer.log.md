# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:18:48+00:00

## timeline

### 20:19:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan resistance is what moves hit rate (cycles 0002+). Capacity, associativity and plain LRU reordering were not the limit.\n- Mid-bucket insertion (around len/4) with gradual transpose-by-one promotion beats LRU. It was reproduced independently from baseline in cycles 0003, 0006 and 0008. Its cost is put-tail shuffling.\n- Real reuse signals beat proxies. The adaptive global hit/miss scheme (0008) and the ghost ring of recent evictions (0010) gave genuine hit-rate gains. Both fit in existing metadata.\n- Removing redundant put-path work (half-bucket shifts, repeated hash/modulo) cut put latency without hurting hit rate (0004).\n\n## Recurring failures\n\n- Get latency has had no real gain in eight cycles. Hashing, arithmetic, item movement, tag/alignment layout and fusion were all ruled out. Every per-hit addition (flag, counter, extra compare or swap) made get slower.\n- Tuning the insertion constant is saturated on both sides: len/3 and len/5 both lost to len/4. Cascaded or longer displacement chains only moved cost.\n- Faster or gated promotion (geometric jump, seen-once gate, position-gated two-slot) removes the frequency filter that transpose-by-one already provides. Hit rate dropped or gained only marginally.\n- Stateless randomization (hash-bit victim choice) and weak proxies (victim position, sampled global ratio, one-entry ghost) carry no reuse signal.\n- Stacking cheap scan-resistance mechanisms (reference bit plus low insertion) did not compound.\n- Alignment and layout tweaks do not reproduce. Put-latency gains with no mechanism keep turning out to be noise.\n- Winner status is repeatedly disconnected from the variants' own insights. Many Winners report losses, so treat Winner labels with suspicion.\n- No variant has grown metadata, and none has been validated on a second workload.\n\n## Open questions\n\n- Can the ghost-ring signal be made cheaper (smaller tags, hash filter, fewer compares)? Would a larger ring, or ghost-guided eviction, add hit rate?\n- Is the get-latency rise seen even when get is untouched a real layout effect or noise? A noise check is needed.\n- Where does get time actually go? Harness or dispatch overhead is untested.\n- Can swap insertion keep its put saving without demoting the displaced entry to the tail?\n- How much of the adaptive gain comes from adaptivity versus fixed mid-insertion mechanics?\n- Do any gains hold on a second workload, and are the Winner statuses beyond noise?\n\n## Per-policy notes\n\n- LRU (baseline): position-only tweaks and partial promotion did not help. Hit rate is insensitive to associativity.\n- CLOCK reference bit: cheap and a modest win alone (0002). It hurts when stacked on low insertion.\n- Frequency counter: marginal gain, with a per-access latency cost. Dropped.\n- LIP/BIP-style low insertion: a clear gain alone (0002). Pushing too deep starves new entries.\n- Mid-insertion plus transpose promotion: the settled baseline. Swap-based len/4 insertion is cheaper on put but loses hit rate, because the swap demotes the mid entry to the tail.\n- Adaptive insertion depth: works with a global hit/miss signal. Put-side proxies and sampled counters failed.\n- Ghost admission or ring: a single-entry ghost hurt. A 4-tag ring of recent evictions gave the cleanest recent hit-rate gain at a put-time cost, and is the current lead.\n"
}
```

### 20:19:05 chat

I wrote `cross-cycle-insights.md` to the output dir. It has the four required sections in order: Recurring wins, Recurring failures, Open questions and Per-policy notes. I regenerated it from the cycle 0001–0010 digest alone, and I didn't read the previous version.

- **What's working:** Scan resistance is what moves hit rate. Mid-bucket insertion with gradual transpose promotion beats LRU and has been reproduced in cycles 0003, 0006 and 0008. Real reuse signals (the cycle 0008 global hit/miss counter and the cycle 0010 ghost ring) beat proxies.
- **What keeps failing:** Get latency has had no real gain in eight cycles. Tuning the insertion constant and faster or gated promotion are exhausted. Stateless randomization and weak proxies carry no reuse signal. Winner status often disagrees with a variant's own insight.
- **Current lead:** The 4-tag ghost ring is the lead. It gave a modest hit-rate gain at a put-time cost, so the open questions ask how to make it cheaper and whether a larger ring or ghost-guided eviction adds more.


---

- finished: 2026-10-20T20:19:05+00:00
- duration: 16.87s
- status: ok
- input_tokens: 4
- output_tokens: 1771
- cache_read_tokens: 41500
- cache_write_tokens: 13431
- cost_usd: 0.076513
