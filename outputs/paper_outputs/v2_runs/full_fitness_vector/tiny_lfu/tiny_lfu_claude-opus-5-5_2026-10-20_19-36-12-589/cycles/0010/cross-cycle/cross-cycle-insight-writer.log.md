# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:50:52+00:00

## timeline

### 20:51:17 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Removing work from the hot path.** The gains that held up all deleted work from the per-op path: cheap unkeyed hashing (0001), the fused register-held sketch increment (0004 and 0006), and the xor-fold finaliser (0010, the first change to lower `get` p50). When the critical path is a serial ALU chain, the finaliser's latency matters more than the number of hash calls.\n- **Shrinking metadata without changing behaviour.** Compact `u32` links (0002), CLOCK in place of LRU links (0006), a region flag packed into a link bit (0005) and `u8` counters (0009) all cut memory with hit rate identical or within noise.\n- **Sketch accuracy with row independence kept.** Conservative update (0003), `u16` counters at twice the width (0004) and single-counting each access (0005) each gave a small hit-rate gain. The returns shrank each time, and this lever now looks used up.\n- **Changing eviction order inside the main region.** GCLOCK's multi-life usage counter (0010) raised hit rate with the admission duel left unchanged. It is the first structural hit-rate gain in the run.\n- **Behaviour-preserving refactors on CLOCK.** These have kept hit rate identical across several cycles, so the base is safe to refactor.\n\n## Recurring failures\n\n- **Aging and decay of any kind.** This covers halving with 4-bit counters (0002), halving with full-width counters plus conservative update (0004), the doorkeeper's all-at-once clear (0008) and 4-bit saturation without aging (0010). Workload-a rewards stable long-term frequency, and decay or a low cap creates ties that the strict `>` rule resolves against newcomers.\n- **Admission windows of any size.** A ~1% window (0005) and a 10% window (0006) both lowered hit rate. Capacity given to unfiltered arrivals goes to one-hit keys.\n- **Changing the duel's inputs.** Tie-favouring `>=` (0003), sampling at the LRU tail (0002) and the colder of two CLOCK candidates (0007) were all flat or negative. The strict single-victim duel is still best.\n- **Blocked or one-cache-line sketch layouts** (0003, 0009). Correlating the rows inflates cold keys' estimates.\n- **Extra fixed per-access work.** Prefetching, extra hash mixes, index recomputation, nibble masking, guard compares and extra struct fields each cost more at the median or tail than the rare-path miss savings they targeted.\n- **Optimising rare paths.** Sweep micro-optimisation and fill-phase map tweaks (0007) changed nothing, because steady state seldom reaches those paths.\n- **Reducing hash count and replacing the map.** Hash-once and identity-keyed maps (0006, 0008) and index-only custom tables (0006) did not beat SwissTable.\n- **Changing sketch footprint as a latency lever.** Widening cost median latency (0008). Shrinking left `get` p50 flat (0009, 0010).\n- **Single p99 readings and unexplained median shifts.** These have repeatedly turned out to be bench drift. A gain counts only if a mechanism explains it and it repeats side by side against the parent.\n\n## Open questions\n\n- Do the xor-fold hash and GCLOCK compose? Would interleaving both usage bits in one word per slot keep the hit-rate gain without the tail regression?\n- Does a 3-bit usage counter add more hit rate, or do longer worst-case sweeps cancel it out?\n- Are there other eviction-order levers inside the main region, now that admission-side levers are used up?\n- How much of `put` p50 and the tails comes from map remove/insert plus the sweep, and can that cost be cut without extra hashing or a larger struct?\n- Is `get` p50 now at its floor for one SwissTable lookup plus one sketch increment?\n\n## Per-policy notes\n\n- **LRU (0001–0005):** Exact recency order gave the duel no useful information. SLRU or window restructuring added cost and gave no hit rate.\n- **CLOCK (0006–0009):** Hit rate matched LRU with smaller metadata. Steady-state sweeps are short because newcomers enter with a clear bit. `put` p99 is sensitive to unbounded sweeps and to work on the rare paths.\n- **GCLOCK (0010):** This is the first policy to raise hit rate by protecting residents that get repeat hits. Its costs are an extra bit-plane read-modify-write and longer sweeps, both of which show up in the tails.\n- **TinyLFU admission, throughout:** The strict `>` duel against a single victim, with non-decaying count-min counters, held across all ten cycles. Admission is decided by low counts as long as the counter cap sits far above the counts steady state reaches: `u8` works, 4-bit does not.\n"
}
```

### 20:51:26 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four required sections in order and was built only from the ten-cycle ledger, with no numbers.

- **Recurring wins:** Cutting work from the hot path paid off most: cheap hashing, the fused increment, and the xor-fold hash, which was the first change to lower `get` p50. Shrinking metadata without changing behaviour also worked, as did sketch accuracy, though its gains have run out. Changing eviction order with GCLOCK is the first structural change to raise hit rate.
- **Recurring failures:** Several approaches kept failing:
  - any kind of aging or decay
  - admission windows of every size tried
  - changes to the inputs of the admission duel
  - sketch layouts that pack a key's counters together
  - extra fixed work on every access
  - optimising paths the workload rarely takes
  - reducing hash count or replacing the map
  - resizing the sketch to cut latency
- **Open questions:** These include whether the xor-fold hash and GCLOCK combine cleanly, whether storing both GCLOCK usage bits in one word removes its extra cost, and whether a 3-bit usage counter helps. Others are what else could change eviction order, whether the map operations and sweep explain the `put` and tail costs, and whether `get` p50 is at its floor.
- **Per-policy notes:** Short notes on LRU, CLOCK, GCLOCK, and the TinyLFU admission rule, which held across all ten cycles as a strict `>` duel against one victim with counters that never decay.


---

- finished: 2026-10-20T20:51:26+00:00
- duration: 34.32s
- status: ok
- input_tokens: 4
- output_tokens: 2290
- cache_read_tokens: 42296
- cache_write_tokens: 15399
- cost_usd: 0.178545
