# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-28T20:29:29+00:00

## timeline

### 20:29:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Shrinking or packing metadata is reliable. The `u32`-index node (c1), in-link segment flags (c2), and padding-byte or `prev`-bit flags (c2, c5) all held the node at its compact size. Fields that add to the node without reusing free bits cost a full word.\n- Cutting the number of hash operations on put is the only latency lever that keeps working. The single-`Entry`, no-clone put (c4) moved put without side effects. Every added hashed structure gave that gain back (c3, c5).\n- SLRU is the strongest hit-rate base. It improved every trace at both sizes (c2) and has been the parent for every cycle since. Representation-only refinements on it change one axis cleanly and leave hit rate unchanged.\n- Gentle, selective second chances help where blunt admission hurts. A one-shot fingerprint reprieve that stays inside probation (c4, c5) was the first history mechanism to lift web_0. It beat a uniformly wider probation (c5).\n- Gating by cache size works. Turning hit-rate mechanisms on only at the small size kept their 16K gains and removed losses at the large size (c5).\n\n## Recurring failures\n\n- Hasher swaps made get no faster (c1, c2) and in one case made it worse. Get p50 seems bound by the map probe and memory, not by hash speed or list work.\n- Relink and branch micro-optimisation on the hit path is used up (c1, c3, c5). Removing list work never lowered get p50. CLOCK-style protected bits only moved the cost into sweeps.\n- Any extra work on the hit path or in the header's hot line raised get p50, even a single AND or mask (c4, c5). Per-cache side structures (ghost sets, sketches) also hurt get without being touched on get (c3, c4).\n- Aggressive or strict admission hurt web_0. Ghost-to-protected was too eager and the miss-only, tie-rejecting TinyLFU gate was too strict (c3).\n- Online hill-climbing of the SLRU split chased noise (c4). Static probation-share retuning helped one trace and hurt the other (c5).\n- GCLOCK with decrement-by-one aging made put sweeps long and its gains were narrow (c2).\n- Policy changes almost always move several axes at once. A hit-rate gain usually comes with a put-path cost.\n\n## Open questions\n\n- Can the reprieve ghost reuse the map's own hash (raw-entry or `hashbrown` API) and run only on the vacant path, to win back the single-probe put cost?\n- Does get p50 recover if `set_protected` is a single OR and `SC_BIT` is cleared at reprieve or eviction time?\n- How much of the per-op timing delta is run-to-run noise? Get p99 swings now look like noise. Repeated paired parent/child runs are needed before trusting changes under a few percent.\n- Map/arena locality and table density or hash quality are still untested levers for get p50.\n- The cause of the get regression from side structures, cache pressure versus benchmark interleaving, is still unknown.\n- web_0 at 131K is still unsolved by any mechanism. It may need a W-TinyLFU-style window, or a sketch that counts hits too and uses a cheap hash.\n\n## Per-policy notes\n\n- SLRU: base since c2, on a `u32` node with the segment flag in a link bit and a single-`Entry` put. Its remaining costs are the promote/demote relinks and per-segment branching on put. Probation looks undersized for web_0 at 16K, but a fixed wider share hurts hm_0.\n- SLRU + fingerprint reprieve: the current best hit-rate variant (c5), size-gated to small caches. Its costs are double hashing on put and a small hit-path bit operation.\n- CLOCK: its lazy promotion shrank metadata, but get p50 was nearly flat and put tail latency was unbounded (c1). As a protected-segment substitute it did not help (c5).\n- GCLOCK: its gain was mostly hm_0 at the large size. Not worth keeping unless aging is moved off the eviction path, for example by bulk halving.\n- Ghost/A1out and TinyLFU gate: full-key ghosts and miss-only, tie-rejecting gates both failed. Any history state must be tiny, cache-friendly, and admit gently.\n"
}
```

### 20:29:54 chat

I wrote a fresh `cross-cycle-insights.md` in the output dir from the ledger digest for cycles 1–5. It uses the four sections in the required order, with no numbers. I did not read the previous version.

**Recurring wins:**
- Making node metadata smaller or packing flags into spare bits.
- Doing fewer hash operations on put. This is the only latency fix that has kept working.
- SLRU as the base policy.
- A gentle second chance for keys that come back, instead of strict admission rules.
- Turning hit-rate mechanisms on only at the small cache size.

**Recurring failures:**
- Changing the hasher.
- Small tweaks to relinking and branching on the hit path.
- Adding any work to the hit path, or adding per-cache side structures. Both made get slower.
- Strict admission rules, which hurt web_0.
- Tuning the SLRU split, either at runtime or with a fixed share.
- GCLOCK with decrement-by-one aging, which made put sweeps long.

**Open questions:**
- Can the ghost reuse the hash the map already computes?
- Does get latency recover with a cheaper `set_protected`?
- How much of the timing movement is noise? Small deltas need repeated paired runs before they can be trusted.
- Memory layout and table density are still untested as ways to speed up get.
- What does web_0 at the large size need? No mechanism has fixed it yet.

**Per-policy notes** cover SLRU, SLRU with the reprieve, CLOCK, GCLOCK, and the ghost/TinyLFU attempts.


---

- finished: 2026-10-28T20:29:54+00:00
- duration: 25.34s
- status: ok
- input_tokens: 4
- output_tokens: 2107
- cache_read_tokens: 38938
- cache_write_tokens: 10586
- cost_usd: 0.134632
