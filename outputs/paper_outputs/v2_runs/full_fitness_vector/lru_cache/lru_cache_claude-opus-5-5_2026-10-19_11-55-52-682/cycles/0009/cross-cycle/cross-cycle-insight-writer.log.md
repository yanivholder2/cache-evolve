# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:05:43+00:00

## timeline

### 10:05:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The safest gains come from changing the representation while keeping eviction order bit-for-bit. Examples: `u32` links, singly-linked SIEVE, the Fx hasher, the packed side array and the VISITED bitset. These gave metadata or latency gains in cycles 0003–0008 without ever costing hit-rate.\n- Policies that quickly demote new keys and lazily promote reused keys beat strict LRU order. The line went CLOCK/SLRU, then SIEVE, then S3-FIFO. A single visited bit was enough, and deeper counters never helped.\n- Ghosts help, but only behind a probationary queue (the S3-FIFO small + main + ghost split). A short, bounded ghost reach works better than a long one, and a ghost hit should be treated as a strong admission signal.\n- The latency levers moved over time. Node layout came first, then hashing (Fx), then where the flags live. Flags written on every hit should sit in a tiny structure addressed by index (the bitset), because the hand and the queue pops read them by index.\n- A structural policy change can fix the put tail. S3-FIFO's O(1) small-queue evictions did more for put p99 than capping the sweep did.\n\n## Recurring failures\n\n- Stacking scan-protection mechanisms didn't work. BIP inside SLRU probation shrank the admission window. The half ghost combined with main-eviction ghosting interfered rather than stacked, since both add selectivity and that dilutes the table.\n- Deeper reuse credit in the queue (GCLOCK 2-bit, SIEVE 2-level counter) and sweep caps both cost hit-rate. The deeper credit also hurt the put tail.\n- Tuning S3-FIFO knobs on workload-a has plateaued for four or more cycles. A wider small queue, a larger or associative ghost, and placing returners at the hand were neutral or negative.\n- Hand-rolled key-less hash tables and values stored in map buckets were slower. Wider or unfiltered buckets make every probe, insert and remove more expensive.\n- VISITED stored in the map bucket sped up get but turned each hand step into a hashed probe, and the put tail regressed badly.\n- The TinyLFU duel run as a speculative hand pass wore down main's VISITED protection and lost on every axis.\n- Every result has been measured on one trace (workload-a) at one size (2048). Each cycle has deferred this check, and it is the main threat to every conclusion.\n\n## Open questions\n\n- Does the S3-FIFO (or SIEVE) lead hold at another cache size or trace? This has been unverified for eight cycles and should come before any further tuning.\n- Would frequency admission work if it only peeked at the victim without destroying state and counted every main hit? Or does workload-a simply not reward admission signals beyond the ghost?\n- Can map buckets shrink below key+`u32` (fingerprint or open addressing into the arena) while keeping cheap tag filtering? The map probe is now the floor for get.\n- Can the put-tail map work be cut to a single probe with hashbrown's entry or raw APIs?\n- Metadata dimensions still ignore heap side structures (ghost, sketch, side arrays, map growth), so per-node wins are overstated.\n- Run-to-run latency drift is about as large as the effects being measured. Parent and variant need to be compared in the same run, with repeated runs.\n\n## Per-policy notes\n\n- **LRU (baseline):** strict recency order lost to every alternative tried on workload-a. Further tweaks to the order it keeps are dropped.\n- **SLRU:** an early winner thanks to segmentation and `u32` links, but head insertion into probation hurt. If revisited, adjust probation size or admission, not insertion position.\n- **CLOCK / GCLOCK:** the relink-free hit path is cheap. Deeper per-slot credit trades put p99 for hit-rate. This line was superseded by SIEVE.\n- **SIEVE:** one queue and one bit came close to a local optimum for signals inside the queue. The singly-linked variant gave free metadata savings. Counter depth and sweep caps are closed.\n- **S3-FIFO (current lineage):** the only policy change since SIEVE to raise hit-rate, and it also helped the put tail. Current parent: Fx hasher, VISITED bitset, ghost of main evictions at half size. Ghost size and inputs are closed, the split is near its optimum around 10%, and further gains probably need a different signal or a different trace.\n"
}
```

### 10:06:05 chat

I wrote `cross-cycle-insights.md` from the cycle 0001–0009 ledger, with the four required sections in order. It has no numbers, and I didn't read the previous version of the file.

- **Recurring wins:**
  - Changes to data layout or hashing that keep the eviction order exactly the same have been the safe way to gain. They were never paid for in hit-rate.
  - Policies that drop new keys quickly and promote reused keys slowly beat strict LRU. The line went from LRU to SIEVE to S3-FIFO.
  - Ghost entries for evicted keys only help when there's a probation queue for them to bypass. A short ghost history works better than a long one.
- **Recurring failures:**
  - Combining two scan or ghost protections made them interfere instead of adding up.
  - Deeper reuse counters and capping the sweep both cost hit-rate.
  - Tuning S3-FIFO's knobs on workload-a has plateaued for four or more cycles.
  - Hand-rolled hash tables, storing values in the map buckets, keeping the visited flag in the bucket, and the TinyLFU duel all lost.
- **Open questions:**
  - Every result so far comes from one trace (workload-a) at one cache size (2048). The file flags this as the biggest threat to every conclusion.
  - The metadata measure doesn't count heap structures such as the ghost table and side arrays, so per-node memory savings are overstated.
  - Speed differences between runs are about as large as the effects being measured. Parent and variant should be timed in the same run.
- **Per-policy notes:** one short entry each for LRU, SLRU, CLOCK/GCLOCK, SIEVE and S3-FIFO. The current S3-FIFO parent combines the Fx hasher, the visited-bit bitset, and a half-size ghost that also records main-queue evictions.


---

- finished: 2026-10-20T10:06:05+00:00
- duration: 22.27s
- status: ok
- input_tokens: 4
- output_tokens: 2258
- cache_read_tokens: 41401
- cache_write_tokens: 14370
- cost_usd: 0.168416
