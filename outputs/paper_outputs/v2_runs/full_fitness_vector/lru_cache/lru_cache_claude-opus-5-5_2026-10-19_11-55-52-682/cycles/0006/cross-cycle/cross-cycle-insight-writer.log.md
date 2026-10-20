# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:22:50+00:00

## timeline

### 14:23:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0006)\n\n## Recurring wins\n\n- **Moving away from strict LRU recency order** improved hit-rate in cycle 0001 with all three alternatives tried. Since then, the hit-rate gains have come from quick demotion of new keys plus lazy promotion of reused keys (SIEVE, cycle 0002) and from a probationary queue with ghost bypass (S3-FIFO, cycle 0005).\n- **Changes to representation alone that keep eviction order the same** have been the most reliable gains (cycles 0003–0006): `u32` links, a singly-linked queue, packed side arrays and key swap. They have never cost hit-rate, and they usually improve metadata.\n- **A hit path that never relinks**, using hands or bits instead of moving nodes, has kept get cheap in every hand-based policy since cycle 0001.\n- **Putting policy state in padding bytes** (`u8` freq, `bool` visited) adds state at no metadata cost.\n- **Cheap hashing (Fx-style) was the main speed lever** once node layout stopped moving per-op-time. It helped get once, and helped the put tail and struct size more (cycles 0004–0005).\n- **Structural policy fixes can also help the tail.** S3-FIFO's O(1) evictions from the small queue improved put p99 even though each miss did more work.\n\n## Recurring failures\n\n- **Stacking scan-protection mechanisms.** BIP inside SLRU probation (cycle 0002) shrank the admission window, so new keys had less chance to earn a second touch.\n- **Tuning in-queue reuse signals has hit a local optimum.** Deeper counters (GCLOCK, a 2-level SIEVE counter) and sweep caps cost hit-rate and/or put-tail latency (cycles 0002–0003).\n- **Ghosts without a probationary queue** (a single-queue SIEVE graft, cycle 0004) lost hit-rate and put latency.\n- **Widening S3-FIFO admission.** A larger small queue and bigger or more associative ghosts (cycle 0006) both took capacity from main and cost hit-rate plus put latency. Main capacity for proven-reuse keys is the scarce resource.\n- **Hand-rolled key-less hash tables** (cycle 0005) lost to hashbrown, badly at the put tail, because buckets cannot be rejected without dereferencing the arena.\n- **Node-shape work for speed** ran out once the hit path no longer relinks. It is now a metadata lever only. Side arrays can even hurt get by splitting a hit across two cache lines (cycle 0006).\n- **Blaming the hand sweep for the put tail** was refuted in cycle 0003. Per-miss map work (hashing, probes, key clones, ghost lookups) dominates.\n\n## Open questions\n\n- **Generalization.** Every result is from workload-a at size 2048. A second cache size or trace has been requested every cycle since cycle 0001 and is still missing. This is the biggest risk to the lineage.\n- Can VISITED stay inline with the value while only the links move out, keeping the memory win without the extra get cache-line touch?\n- Would a *smaller* ghost or softer ghost admission help? Softer admission could mean entering main unvisited, behind the hand. Separately, would split points just below or around 10% (1/8, 1/12) beat the current configuration?\n- Can hashbrown's entry or raw-table API give a single-probe miss path, and does that lower put p99 on S3-FIFO?\n- Metadata accounting still misses heap side structures (ghost table, side arrays, `Vec` headers). Per-node wins overstate the real savings.\n- How much of the reported speed gains come from benchmark noise? This has been flagged since cycle 0001 and never quantified.\n\n## Per-policy notes\n\n- **LRU (baseline):** order-only tweaks were dropped after cycle 0001.\n- **SLRU:** won in cycle 0001 through `u32`-compact nodes and second-touch promotion. Adding BIP insertion to probation hurt it. If revisited, tune the probation size or admission, not the insertion position.\n- **CLOCK / GCLOCK:** the second-chance bit gained hit-rate unexpectedly in cycle 0001. A deeper credit counter buys some hit-rate but reliably hurts the put tail through longer sweeps.\n- **BIP:** helped as a standalone insertion change in cycle 0001 and interfered when layered on SLRU. Not pursued further.\n- **SIEVE:** one queue and one bit matched or beat multi-segment policies. The lineage refined it through a singly-linked layout, the Fx hasher and a packed side array. Its in-queue knobs are exhausted. It is the base for the S3-FIFO main queue.\n- **S3-FIFO (current hit-rate parent, cycle 0005 v003):** a small FIFO, a main SIEVE and a direct-mapped fingerprint ghost. This is the first policy change since SIEVE to raise hit-rate, and the gain was modest. The ghost's bounded, collision-limited reach works as a useful filter. The put path is limited by hashing and ghost probes on each miss. Its side-array variant (cycle 0006 v002) saves memory at a get cost.\n"
}
```

### 14:23:15 chat

I've written `cross-cycle-insights.md` from the ledger of cycles 0001–0006, using the four required sections in order.

The main takeaways for the next cycles:

- **Biggest risk:** every result so far comes from one trace (workload-a) at one cache size (2048). A second size or trace has been asked for every cycle since 0001 and still hasn't been run.
- **Current best for hit-rate:** S3-FIFO from cycle 0005 v003, which has a small queue, a main SIEVE queue and a list of recently evicted keys (a "ghost"). It was the first policy change since SIEVE to raise hit-rate, though the gain was modest. Cycle 0006 suggests its settings are now near a local best: a bigger small queue or a bigger ghost both lowered hit-rate.
- **Most reliable gains:** changes to memory layout that keep the eviction order the same have never cost hit-rate. A cheaper hash function was the last change that clearly sped things up.
- **Approaches that keep failing:** stacking scan-protection mechanisms, tuning SIEVE's per-entry counters or sweep, adding a ghost without a small queue, and hand-rolled hash tables.
- **The put-tail latency** comes from map and ghost work on each miss, not from the eviction sweep. Cycle 0003 ruled out the sweep.
- **Memory figures are still incomplete:** heap structures like the ghost table and side arrays aren't counted, so the per-entry memory wins look bigger than they are.

The file also has short notes per policy (LRU, SLRU, CLOCK/GCLOCK, BIP, SIEVE, S3-FIFO).

Files are in /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0006/cross-cycle:
- cross-cycle-insights.md


---

- finished: 2026-10-19T14:23:15+00:00
- duration: 25.50s
- status: ok
- input_tokens: 4
- output_tokens: 2531
- cache_read_tokens: 38699
- cache_write_tokens: 11940
- cost_usd: 0.153896
