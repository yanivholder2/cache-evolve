# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:18:19+00:00

## timeline

### 12:18:33 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Leaving strict LRU ordering behind paid off from the start (cycle 1). The hand-based, relink-free hit path that came out of that (CLOCK → SIEVE) has kept get cheap in every cycle since.\n- SIEVE has been the strongest policy line since cycle 2. It demotes new keys quickly and promotes reused keys lazily, using one queue and one visited bit. It matched or beat the multi-segment SLRU on hit-rate.\n- Changes that alter only the representation and keep eviction order bit-for-bit have never regressed (cycles 3–4). Examples are a singly-linked queue with a predecessor-carrying hand, `u32` links/indices, a packed side array, and padding reuse for policy bits. This is the most reliable way to improve metadata.\n- Node compaction helped speed only while hits still relinked (cycle 1). On SIEVE it is purely a memory lever.\n- The latency lever turned out to be hashing and map work on the miss path (cycle 4: a faster hasher with `u32` map values). That fits cycle 3's finding that put p99 does not come from the hand.\n\n## Recurring failures\n\n- Stacked scan or loop protection does not add up. BIP inside SLRU probation shrank the second-touch window and cost hit-rate (cycle 2).\n- Changing how the hand treats queued nodes keeps costing hit-rate. Deeper per-slot credit (GCLOCK, 2-level SIEVE counter) also made the put tail worse, and capping the sweep evicted real reuse without lowering put p99 (cycles 2–3). SIEVE's single bit looks like a local optimum for reuse signals kept inside the queue.\n- A ghost/history signal grafted onto a single queue lost hit-rate and put latency (cycle 4). With no probationary queue to bypass, it only kept long-reuse-distance keys at the expense of better residents, and it added hashing on every miss.\n- Explaining the put tail by sweep length was wrong. It was diagnosed in cycle 2 and refuted in cycle 3.\n\n## Open questions\n\n- Does a full S3-FIFO-style split (small probationary FIFO, then main SIEVE, plus a ghost queue) gain the hit-rate that the single-queue ghost graft lost?\n- How much put p99 is left after cutting probes per miss: entry-style remove/insert, one hash reused across the probe, no key clone?\n- Do the cheaper hasher and the packed side array stack cleanly, given the extra memory touch per hit?\n- Every result so far comes from the LIRS workload-a trace at a single cache size (2048). The rankings, and SIEVE's lead in particular, have been unverified for four cycles. This is now the largest overfitting risk.\n- The metadata metric under-counts heap tables and side arrays. Per-entry side-array bytes need to be included for comparisons to stay fair.\n\n## Per-policy notes\n\n- **LRU (baseline):** further tweaks to its order are not worth pursuing (dropped in cycle 1).\n- **SLRU:** improved hit-rate and nodes over the baseline. Head insertion into probation hurt it. If revisited, tune probation size or admission instead of insertion position.\n- **BIP:** improved on its own against loops, with flat metadata. Inside SLRU it worked against the admission window.\n- **CLOCK / GCLOCK:** CLOCK gained on all axes in cycle 1. The 2-bit GCLOCK traded put-tail latency for hit-rate, and further single-bit CLOCK tweaks were dropped.\n- **SIEVE:** the current lineage. Singly-linked and Fx-hashed SIEVE (cycle 4 v002) is the parent. Changes to counter depth, sweep caps and single-queue ghosts are exhausted. The next steps are miss-path map work, the side-array combination, and an S3-FIFO-style probationary/ghost structure.\n"
}
```

### 12:18:39 chat

I've written `cross-cycle-insights.md` in the cycle 0004 output directory, built fresh from the ledger digest for cycles 1–4, with the four required sections in order.

- **Recurring wins:** SIEVE has been the strongest policy since cycle 2. Changes that only alter the data layout and keep eviction order identical have never made anything worse. Speed now comes from cheaper hashing and less map work when a key is missing from the cache, not from node layout.
- **Recurring failures:** Stacking two scan-protection mechanisms hurts hit-rate, as BIP inside SLRU did in cycle 2. Changing how SIEVE's eviction pointer treats queued entries has run out of value: extra counters and sweep caps both lost hit-rate. A history of evicted keys added to a single queue lost on hit-rate and latency.
- **Open questions:** The biggest risk is overfitting. Every result comes from one trace (workload-a) at one cache size (2048), and that is still unchecked after four cycles. The memory metric also under-counts side arrays and heap tables. The other questions are a full S3-FIFO-style split, cutting map lookups on a miss, and whether the cycle 4 hasher and side-array changes combine cleanly.
- **Per-policy notes:** These cover LRU, SLRU, BIP, CLOCK/GCLOCK and SIEVE. The current parent is singly-linked SIEVE with the faster hasher (cycle 4 v002).


---

- finished: 2026-10-19T12:18:39+00:00
- duration: 19.65s
- status: ok
- input_tokens: 4
- output_tokens: 1941
- cache_read_tokens: 36625
- cache_write_tokens: 9407
- cost_usd: 0.121417
