# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T09:59:09+00:00

## timeline

### 09:59:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes to representation alone, with eviction order identical bit-for-bit, have been the most reliable gains (cycles 3–8). They almost never cost hit-rate. Since about cycle 6 they have stopped being free on latency, so check `get` and the `put` tail on each one.\n- Leaving strict LRU order is the main hit-rate lever. The progression has been LRU → SLRU/CLOCK/BIP → SIEVE (quick demotion plus lazy promotion with one bit) → S3-FIFO (small FIFO + main SIEVE + ghost). Each step that paid off added a new structural signal, not deeper per-entry counters.\n- A ghost of evicted keys pays off only when a probationary queue sits in front of main. On a single queue it lost (cycle 4). Inside S3-FIFO it gained (cycle 5), and the gain held across node layouts.\n- Keeping the ghost's reach short works better than extending it. Direct-mapped collisions and a half-size ghost act as a useful recency filter (cycles 6–8).\n- Hit paths that never relink, combined with compact `u32`-index nodes, keep `get` cheap and shrink metadata.\n- Cheap hashing (an Fx-style hasher on hashbrown) was the biggest single latency lever, especially for the `put` tail on misses. It now mainly benefits the miss path.\n- Flags written on every hit belong in a tiny structure addressed by index (the VISITED bitset). That keeps hand sweeps cheap and avoids most of the side-array `get` penalty.\n- Structural policy changes can lower the `put` tail. S3-FIFO's O(1) small-queue evictions did this despite doing more work per miss.\n\n## Recurring failures\n\n- Stacking scan-protection mechanisms. BIP inside SLRU probation shrank the window for a second touch and lost hit-rate.\n- Deeper in-queue reuse credit (GCLOCK, 2-level SIEVE counter) hurt the `put` tail and did not gain hit-rate.\n- Sweep caps did not fix the `put` tail. Long sweeps were not the cause, and evicting runs of visited nodes early costs hit-rate.\n- Tuning S3-FIFO knobs on workload-a (wider small queue, larger or associative ghost, returners placed at the hand) gives noise-sized or negative moves. It is a plateau, and main capacity is the scarce resource.\n- Weakening admission after a ghost hit backfires. A ghost hit is a strong reuse signal, so tune recall, not where returners are placed.\n- Hand-rolled key-less hash tables lose to hashbrown, because they can't reject a bucket without dereferencing the arena. The loss is sharp at the `put` tail.\n- Putting flags where only the key-addressed readers can reach them (VISITED in the map bucket) makes every hand step a hash probe, so the `put` tail regresses.\n- Node-shape work aimed at speed stopped paying once the hit path stopped relinking. Layout now moves memory only.\n\n## Open questions\n\n- Does any of the lineage's ranking (SIEVE over SLRU/CLOCK, S3-FIFO over SIEVE, ghost sizing) hold at another cache size or trace? This has gone unverified for every cycle so far and is the largest gap.\n- Can latency deltas be trusted? Unchanged code paths have moved by uniform proportions across runs. Latency should be compared within the same run as the parent, with repeated runs.\n- Do the half ghost and main-eviction ghosting stack, or does a smaller table get diluted by main victims?\n- What structurally different idea could add hit-rate beyond the S3-FIFO plateau? Candidates: frequency-aware admission (TinyLFU-style), adaptive split sizing, LIRS/ARC-style non-resident history.\n- Can the miss path get down to one probe through hashbrown's entry or raw APIs without losing tag filtering?\n- Metadata accounting ignores heap side structures (ghost tables, side arrays, bitsets, `Vec` headers). Per-node wins overstate the real savings until these are counted per entry.\n\n## Per-policy notes\n\n- **LRU (baseline):** Its strict recency order is a liability on workload-a. Do not tweak it further.\n- **SLRU:** It gained over LRU and `u32` links helped it. Insertion position inside probation was the wrong knob. If revisited, change probation sizing or admission instead.\n- **CLOCK / GCLOCK:** A hit path with no relinking helps `get`. Deeper per-slot credit costs the `put` tail with no hit-rate payoff. SIEVE superseded it.\n- **BIP:** It worked as a standalone departure from LRU (cycle 1) but was counterproductive inside SLRU.\n- **SIEVE:** One visited bit looks like a local optimum for in-queue signals. A singly-linked queue and packed side arrays are safe memory wins. A ghost grafted onto a single queue lost.\n- **S3-FIFO (current lineage):** Small FIFO + main SIEVE + direct-mapped fingerprint ghost, using an Fx hasher, a VISITED bitset and a half-size ghost. It holds the best hit-rate and has a healthy `put` tail. Its admission and ghost knobs have plateaued on workload-a. Main-eviction ghosting is marginally positive.\n"
}
```

### 09:59:35 chat

I wrote `cross-cycle-insights.md` in the cycle 0008 cross-cycle folder. It has the four required sections in order and was built only from the ledger for cycles 1–8.

- **Recurring wins:**
  - Changes that only alter data layout, without changing which entry gets evicted, have been the safest gains. Since about cycle 6 they have started to cost some latency.
  - Hit-rate improved each time the policy gained a new structure: LRU → SLRU/CLOCK/BIP → SIEVE → S3-FIFO.
  - Remembering recently evicted keys (a "ghost" table) only helped once S3-FIFO added a probation queue in front of the main cache. Keeping that table small helped more than making it larger.
  - The cheaper hash function was the biggest single latency gain.
  - Storing the per-entry "visited" flag in a small bit array kept the eviction scan cheap.
- **Recurring failures:**
  - Stacking two scan-protection mechanisms lost hit-rate.
  - Giving entries more reuse credit inside the queue, or capping the eviction sweep, did not help.
  - S3-FIFO's settings on workload-a have plateaued: recent tweaks moved results by noise-sized amounts or made them worse.
  - The hand-written hash table and the move of the "visited" flag into the map both slowed the `put` tail.
- **Open questions:**
  - No result has been checked at a second cache size or on a second trace in any cycle. This is the largest gap.
  - Latency differences between runs can't be trusted yet, because untouched code paths shift between runs.
  - Further hit-rate gains probably need a structurally new idea; the file lists frequency-based admission and keeping history of evicted keys as candidates.
  - The memory metric doesn't count heap side structures, so per-node savings look bigger than they are.
- **Per-policy notes:** one line each for LRU, SLRU, CLOCK/GCLOCK, BIP, SIEVE and the current S3-FIFO line.


---

- finished: 2026-10-20T09:59:35+00:00
- duration: 25.94s
- status: ok
- input_tokens: 4
- output_tokens: 2626
- cache_read_tokens: 40475
- cache_write_tokens: 13756
- cost_usd: 0.170679
