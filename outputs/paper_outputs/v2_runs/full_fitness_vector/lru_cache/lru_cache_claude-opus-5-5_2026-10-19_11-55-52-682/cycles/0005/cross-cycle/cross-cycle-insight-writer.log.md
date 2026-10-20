# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:24:43+00:00

## timeline

### 12:25:03 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Changes to representation alone have been the safest gains (cycles 1–5).** Narrowing links to `u32`, removing the back-link, packing state into side arrays, reusing padding bytes and swapping the Fx hasher all kept eviction order identical. Each one improved metadata or latency without costing hit-rate.\n- **Moving away from strict LRU order pays off on hit-rate.** The approach that works: demote new keys quickly and promote reused keys lazily. It started with SLRU, CLOCK and BIP (cycle 1), then SIEVE (cycle 2), then S3-FIFO (cycle 5). Each step forward came from handling one-hit or scan keys better, not from deeper reuse counters.\n- **A hit path that never relinks keeps get cheap.** Every hand-based policy (CLOCK, GCLOCK, SIEVE, S3-FIFO) kept get latency low. Once relinking was gone, get latency depended on how many separate memory locations a hit touches, not on node size.\n- **The levers for latency moved over time.** First node layout (cycle 1), then hashing (cycle 4), then structural O(1) small-queue eviction (cycle 5). Each lever ran out once it had been used. After that, the next gain came from a different part of the path.\n- **Ghost or history signals work only with a probationary queue to bypass.** The S3-FIFO split (small FIFO + main SIEVE + ghost) gave the first hit-rate gain in several cycles and did not cost latency.\n\n## Recurring failures\n\n- **Stacking scan-protection mechanisms.** BIP inside SLRU probation (cycle 2) shrank the window in which new keys can earn a second touch, and hit-rate dropped. These mechanisms overlap. They do not add up.\n- **Tuning how the hand treats queued nodes.** Deeper per-slot credit (GCLOCK cycle 2, the 2-level SIEVE counter cycle 3) and sweep caps (cycle 3) both cost hit-rate. Deeper credit also made the put tail worse. SIEVE's single visited bit looks like a local optimum.\n- **Blaming long hand sweeps for the put tail.** Capping the sweep did not lower put p99 (cycle 3). The tail came mostly from map work on the miss path. Hashing changes and structural eviction changes have moved it. Hand tweaks have not.\n- **Adding ghosts to a single queue.** In cycle 4, ghosts on a single queue extended the stay of keys whose reuse distance was too long. Those keys pushed out better residents and added hashing on every miss.\n- **Hand-rolled key-less hash tables.** In cycle 5, a hand-rolled table could not reject a bucket without dereferencing the arena. Backward-shift deletion made clusters expensive. hashbrown's tag filtering beats it.\n- **Node-shape work aimed at speed** after relinking was removed. Since cycle 3 it has only saved memory.\n\n## Open questions\n\n- **Generalization:** every result so far comes from one workload, workload-a, at size 2048. Testing a second size or trace has been recommended for five cycles and has never been done. This matters most now that a policy change (S3-FIFO) has won.\n- **S3-FIFO knobs:** how much hit-rate is left in the small-queue ratio and ghost size, and in whether main-queue evictions should feed the ghost? The modest gain suggests workload-a has limited one-hit traffic.\n- **A single-probe miss path:** can hashbrown's entry or raw-table API give one hash per miss while keeping tag filtering?\n- **Hit-path locality:** can the visited bit sit next to the data a hit already reads, so a hit touches one memory location instead of two?\n- **Do the remaining gains stack?** Specifically Fx hashing, the side array, swapping the key on eviction, and S3-FIFO. Fx and the side array did not stack on get latency in cycle 5.\n- **Metadata accounting:** heap memory for ghosts and side arrays, and index density, are invisible to the per-node metric. Judgments about metadata need bytes counted per entry.\n\n## Per-policy notes\n\n- **LRU (baseline):** strict recency order is a liability on workload-a. It has been dropped as a direction since cycle 1.\n- **SLRU:** second-touch promotion and `u32` links helped in cycle 1. Head insertion into probation hurt in cycle 2. If SLRU is revisited, change the probation size or admission, not the insertion position.\n- **CLOCK / GCLOCK:** a hit path with no relinking and a smaller node won in cycle 1. GCLOCK's 2-bit credit bought hit-rate at a visible put-p99 cost in cycle 2. Further CLOCK tweaks have been dropped.\n- **BIP:** works alone as a loop-resistant insertion policy (cycle 1). It works against admission windows when layered inside SLRU.\n- **SIEVE:** the best single-queue policy, with one bit and quick demotion of new keys. The singly-linked version with a side-array layout and Fx hashing is its leanest form. Changes to counters, sweep caps and single-queue ghosts all regressed.\n- **S3-FIFO (small FIFO + main SIEVE + fingerprint ghost):** the current parent. It is the first policy change to improve both hit-rate and the put tail. Its ghost heap is not yet counted in metadata. Its knobs are untuned, and it has not been verified beyond workload-a at size 2048.\n"
}
```

### 12:25:09 chat

I've written `cross-cycle-insights.md` to the cycle 0005 cross-cycle output directory. It uses the four required sections, built only from the ledger for cycles 1–5.

- **Recurring wins:** Changing how the data is stored without changing eviction order has been the safest gain, across all five cycles. Hit-rate gains came from dropping new keys quickly and promoting reused keys lazily, going SLRU/CLOCK/BIP → SIEVE → S3-FIFO. The thing that cut latency kept changing: node layout, then hashing, then O(1) eviction from a small queue. Remembering evicted keys only helped once there was a separate trial queue for new keys to skip.
- **Recurring failures:**
  - Layering scan protections (BIP inside SLRU) cost hit-rate.
  - Deeper reuse counters and sweep caps cost hit-rate, and deeper counters also slowed the put tail.
  - Blaming long sweeps for the put tail was wrong; most of it came from hash-table work on misses.
  - Remembering evicted keys on a single queue cost hit-rate and put latency.
  - The hand-rolled hash table was slower than hashbrown, sharply at the put tail.
  - Changing node shape to gain speed has stopped helping.
- **Open questions:** Every result still comes from one workload (workload-a) at one cache size (2048). Testing another size or trace has been recommended every cycle and never done. Also open: tuning S3-FIFO's settings, checking a miss would need only one hash-table lookup, cutting the memory locations a hit touches, whether the remaining gains add up, and memory-use figures that miss the evicted-key table and side arrays.
- **Per-policy notes:** Short notes for LRU, SLRU, CLOCK/GCLOCK, BIP, SIEVE and S3-FIFO, which is the current parent.


---

- finished: 2026-10-19T12:25:09+00:00
- duration: 26.54s
- status: ok
- input_tokens: 4
- output_tokens: 2631
- cache_read_tokens: 37729
- cache_write_tokens: 11063
- cost_usd: 0.148686
