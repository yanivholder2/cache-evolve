# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:06:55+00:00

## timeline

### 20:07:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Compact `u32`-index nodes are a safe base. They change no policy behaviour, shrink metadata with no side effects, and policy variants built on them keep the size savings.\n- Storing policy state in bits the node already has works. In two cycles, a segment flag in a link's high bit and a counter in the padding byte left the node at its compact size. Any new field that doesn't reuse existing space costs a full word, because the node has no free padding.\n- SLRU-style segmentation gave the most consistent hit-rate gains on the MSR block traces. In cycle 2 the compact SLRU improved every trace at both cache sizes, more broadly than CLOCK or GCLOCK did.\n- Changes to node representation alone move one axis cleanly. Policy changes move several axes at once, with mixed results.\n\n## Recurring failures\n\n- Using the hash map to cut get latency failed twice. Swapping in a cheaper or faster hasher didn't help, and once made get worse. The probe looks bound by memory and sensitive to how hashes spread structured block keys (inferred, since probe lengths aren't measured).\n- Changing the list or nodes doesn't speed up get. Removing relinking and compacting nodes both left get p50 roughly flat, so the cost of a hit is not in list maintenance.\n- Every frequency- or scan-aware policy has paid for its hit-rate gains on the put/eviction path. In SLRU the cost is the double relink and per-segment branching. In CLOCK and GCLOCK it is long sweeps, made worse by decrement-by-one aging.\n- No policy tried so far fixes web_0 at the large size, whether recency segmentation, a single reference bit or small counters.\n\n## Open questions\n\n- Can SLRU's put cost be cut? Options: avoid the second relink on promotion, give updates to existing keys a fast path, or remove the per-segment head/tail branching.\n- Would an adaptive or larger probation share fix SLRU at the small cache size?\n- What does web_0 at the large size need? Candidates are ghost/history entries (an ARC/LIRS-style approach) or an adaptive split.\n- Would table density or better hash mixing move get p50 where hash speed did not? Probe lengths need to be measured first.\n- Can CLOCK-family sweeps be bounded or amortised, for example with a capped sweep or bulk-halving aging moved off the eviction path, while keeping their median gains?\n- Was GCLOCK's get slowdown real, or an effect of long put sweeps interleaved with gets in the benchmark?\n\n## Per-policy notes\n\n- LRU baseline / representation: `u32` link compaction is the standard layout for later variants and leaves hit rate exactly unchanged. Further compaction is not a speed lever.\n- SLRU: it is the strongest hit-rate policy so far. Its first version paid in metadata and latency. Its second, flag-packed on compact nodes, removed the metadata penalty and limited the remaining cost to put. It is the current default base.\n- CLOCK: lazy promotion shrank metadata and helped hm_0, but its effect on hit rate depends on cache size and trace (slightly worse on web_0 at the large size). Its put tail latency is unbounded because of the sweep.\n- GCLOCK: the counter fits in the padding byte, but its hit-rate gain was mostly hm_0 at the large size. Decrement-by-one aging inflates the sweep, so drop it unless aging moves off the eviction path.\n"
}
```

### 20:07:15 chat

I wrote `cross-cycle-insights.md` in the cycle 0002 output dir. It uses the four fixed sections in order and draws only on the two-cycle ledger digest.

- **Recurring wins:** Compact `u32`-index nodes are a safe default base. Storing policy state in bits the node already has (a flag in a link's high bit, a counter in the padding byte) worked in both cycles. SLRU-style segmentation gave the broadest hit-rate gains.
- **Recurring failures:** Using the hash map to cut get latency failed twice, and the second time get got worse. Changing the list or nodes never moved get p50. Every frequency- or scan-aware policy paid for its hit-rate gains on the put/eviction path. Nothing tried so far fixes web_0 at the large cache size.
- **Open questions:** Cutting SLRU's put cost, sizing its probation share at the small cache size, and what web_0 at the large size needs (ghost/history entries or an adaptive split). Also whether table density or hash quality would move get, which needs probe-length measurement first. Whether CLOCK-family sweeps can be bounded. Whether GCLOCK's get slowdown was a benchmark artifact.
- **Per-policy notes:** One short entry each for the baseline/representation work, SLRU (the current default base), CLOCK and GCLOCK. GCLOCK is marked to drop unless its aging moves off the eviction path.

The file has no numbers, since the ledger doesn't give any.


---

- finished: 2026-10-28T20:07:16+00:00
- duration: 20.52s
- status: ok
- input_tokens: 4
- output_tokens: 1785
- cache_read_tokens: 36014
- cache_write_tokens: 7370
- cost_usd: 0.101879
