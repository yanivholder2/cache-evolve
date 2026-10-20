# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:11:31+00:00

## timeline

### 10:11:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Changes that only alter representation and keep eviction order bit-for-bit have been the safest gains in every cycle from 0003 to 0008. Examples are the singly-linked SIEVE, `u32` links and indices, side arrays, the VISITED bitset and the Fx hasher. They reliably improve metadata, and sometimes latency, with no hit-rate risk.\n- Policy changes that pay off are structural, not knob tweaks:\n  - Moving away from strict LRU (0001).\n  - SIEVE's quick demotion of new keys plus lazy promotion (0002).\n  - The S3-FIFO small/main/ghost split (0005), which was also the only policy change to improve the put tail.\n- Ghost entries help only when a probationary queue exists for a ghost hit to bypass. A ghost with limited reach (direct-mapped, half-size) works as a useful filter. Shortening its reach has held up as a small gain across two layouts.\n- Put flags that are written on every hit where the reader can reach them cheaply. The SIEVE hand and queue pops reach nodes by index, so index-addressed compact storage (the bitset) has been the best overall trade.\n- A relink-free hit path combined with cheap hashing (Fx) moved latency more than node layout did. Once the hit path stopped relinking, layout became a memory lever only.\n\n## Recurring failures\n\n- Stacking scan-protection or selectivity mechanisms that overlap. Examples are BIP inside SLRU probation, and the half ghost combined with main-eviction ghosting. They interfere rather than add up.\n- Tuning in-queue reuse signals: deeper counters (GCLOCK, a 2-level visited counter) and sweep caps. These cost hit-rate or tail latency. A single VISITED bit is a local optimum.\n- Changing admission strength for ghost returners. Placing them at the hand (0007) and pre-setting VISITED on them (0010) both lost hit-rate. Plain tail insertion with VISITED clear is bracketed as optimal.\n- Anything that takes effective capacity from main. A wider small queue, a larger or associative ghost, an adaptive split driven by raw counts, and returner credit all lost hit-rate. Main capacity for keys with proven reuse is the scarce resource.\n- Hand-rolled or widened hash tables: the key-less custom index, values stored in buckets, and VISITED stored in the map bucket. Each lost to hashbrown's tag filtering, or made probes or hand steps more expensive.\n- The TinyLFU duel as specified. It ran the SIEVE hand speculatively, wore down VISITED protection and miscounted hot keys.\n- Hash-mixing tweaks. Keys already spread well, so the remaining cost is memory touches.\n- Single-queue ghost grafts. Without a probationary queue to bypass, ghost hits pushed out better residents.\n\n## Open questions\n\n- The top unresolved threat: does any of the SIEVE/S3-FIFO lineage hold up at a second cache size or on another trace? It has been unverified for about nine cycles, and the last several Continuations asked for this check.\n- Latency attribution is unreliable. Drift between runs is as large as the effects being measured. Compare against the parent in the same run, with repeated runs.\n- Metadata accounting does not count heap structures: the ghost table, side arrays, the sketch and map growth. Wins counted per node overstate the real savings.\n- Should promoted small keys keep the VISITED bit they earned when they enter main?\n- Would ARC-style adaptation work if it were normalized by each ghost source's population? Or does workload-a simply have no phases to adapt to?\n- Could frequency admission help if the victim were only peeked at, without disturbing it, and every hit were counted faithfully? Is that only worth trying on a different trace?\n- Could a denser index, such as a fingerprint-indexed or open-addressed table into the arena, cut the number of cache lines per probe? The map probe is now the floor for `get`.\n\n## Per-policy notes\n\n- **LRU / SLRU / BIP:** Strict recency order is a liability on workload-a. SLRU probation already isolates keys seen only once, so BIP on top is redundant and harmful. If SLRU is revisited, change the sizing or admission, not the insertion position.\n- **CLOCK / GCLOCK:** Removing relinking on a hit is cheap and good. Deeper per-slot credit makes the put tail worse through long hand sweeps.\n- **SIEVE:** One queue and one bit gave the best hit-rate among single-queue policies. Neither counter depth nor sweep caps help. Its put tail came from map work on the miss path, not from the hand. The singly-linked form is a free metadata win.\n- **S3-FIFO (current lineage):** The current parent is cycle 0009 v001: Fx hashing, the VISITED bitset, a half-size ghost and main-eviction ghosting. The admission and ghost knobs are a flat plateau on workload-a, and ghost tuning is closed. Further hit-rate gains probably need a new structural signal or a new workload. For latency, target narrower or cheaper map probes.\n"
}
```

### 10:11:56 chat

I've written `cross-cycle-insights.md` with the four required sections, drawn from the ledger for cycles 0001–0010. I didn't read the previous version.

The most important point for future cycles is a warning. No cycle has tested the SIEVE / S3-FIFO line at a second cache size or on another trace, so every conclusion rests on workload-a at one size. That check has been missing for about nine cycles, even though several cycles asked for it. Two smaller caveats are listed as open questions:
- **Latency results are unreliable:** differences between runs are about as large as the effects being measured.
- **Memory numbers overstate savings:** the metadata dimension doesn't count heap structures such as the ghost table and side arrays.

The four sections cover:
- **Recurring wins:** changes to data layout that keep eviction order exactly the same (smaller nodes, compact flags, a faster hasher) have been the safest gains. On the policy side, only big structural steps have paid off: leaving strict LRU, then SIEVE, then the S3-FIFO split with a ghost of short reach.
- **Recurring failures:** stacking overlapping scan protections, deeper reuse counters, giving ghost returners extra credit, anything that takes room from the main queue, custom hash tables, the TinyLFU duel, and hash-mixing tweaks.
- **Open questions:** the untested-workload problem above, then the unreliable latency and incomplete memory accounting, plus a few remaining ideas. These are letting promoted keys keep their visited bit, adaptive queue sizing adjusted for how often each queue evicts, frequency admission that doesn't disturb the queue, and a denser hash index.
- **Per-policy notes:** grouped as LRU/SLRU/BIP, CLOCK/GCLOCK, SIEVE, and S3-FIFO. The S3-FIFO notes name cycle 0009 v001 as the current parent and treat ghost tuning as closed.


---

- finished: 2026-10-20T10:11:56+00:00
- duration: 25.24s
- status: ok
- input_tokens: 4
- output_tokens: 2520
- cache_read_tokens: 42308
- cache_write_tokens: 15513
- cost_usd: 0.182982
