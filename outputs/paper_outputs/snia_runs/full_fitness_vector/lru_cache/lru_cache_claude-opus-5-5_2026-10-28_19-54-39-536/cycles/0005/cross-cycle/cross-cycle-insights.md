## Recurring wins

- Shrinking or packing metadata is reliable. The `u32`-index node (c1), in-link segment flags (c2), and padding-byte or `prev`-bit flags (c2, c5) all held the node at its compact size. Fields that add to the node without reusing free bits cost a full word.
- Cutting the number of hash operations on put is the only latency lever that keeps working. The single-`Entry`, no-clone put (c4) moved put without side effects. Every added hashed structure gave that gain back (c3, c5).
- SLRU is the strongest hit-rate base. It improved every trace at both sizes (c2) and has been the parent for every cycle since. Representation-only refinements on it change one axis cleanly and leave hit rate unchanged.
- Gentle, selective second chances help where blunt admission hurts. A one-shot fingerprint reprieve that stays inside probation (c4, c5) was the first history mechanism to lift web_0. It beat a uniformly wider probation (c5).
- Gating by cache size works. Turning hit-rate mechanisms on only at the small size kept their 16K gains and removed losses at the large size (c5).

## Recurring failures

- Hasher swaps made get no faster (c1, c2) and in one case made it worse. Get p50 seems bound by the map probe and memory, not by hash speed or list work.
- Relink and branch micro-optimisation on the hit path is used up (c1, c3, c5). Removing list work never lowered get p50. CLOCK-style protected bits only moved the cost into sweeps.
- Any extra work on the hit path or in the header's hot line raised get p50, even a single AND or mask (c4, c5). Per-cache side structures (ghost sets, sketches) also hurt get without being touched on get (c3, c4).
- Aggressive or strict admission hurt web_0. Ghost-to-protected was too eager and the miss-only, tie-rejecting TinyLFU gate was too strict (c3).
- Online hill-climbing of the SLRU split chased noise (c4). Static probation-share retuning helped one trace and hurt the other (c5).
- GCLOCK with decrement-by-one aging made put sweeps long and its gains were narrow (c2).
- Policy changes almost always move several axes at once. A hit-rate gain usually comes with a put-path cost.

## Open questions

- Can the reprieve ghost reuse the map's own hash (raw-entry or `hashbrown` API) and run only on the vacant path, to win back the single-probe put cost?
- Does get p50 recover if `set_protected` is a single OR and `SC_BIT` is cleared at reprieve or eviction time?
- How much of the per-op timing delta is run-to-run noise? Get p99 swings now look like noise. Repeated paired parent/child runs are needed before trusting changes under a few percent.
- Map/arena locality and table density or hash quality are still untested levers for get p50.
- The cause of the get regression from side structures, cache pressure versus benchmark interleaving, is still unknown.
- web_0 at 131K is still unsolved by any mechanism. It may need a W-TinyLFU-style window, or a sketch that counts hits too and uses a cheap hash.

## Per-policy notes

- SLRU: base since c2, on a `u32` node with the segment flag in a link bit and a single-`Entry` put. Its remaining costs are the promote/demote relinks and per-segment branching on put. Probation looks undersized for web_0 at 16K, but a fixed wider share hurts hm_0.
- SLRU + fingerprint reprieve: the current best hit-rate variant (c5), size-gated to small caches. Its costs are double hashing on put and a small hit-path bit operation.
- CLOCK: its lazy promotion shrank metadata, but get p50 was nearly flat and put tail latency was unbounded (c1). As a protected-segment substitute it did not help (c5).
- GCLOCK: its gain was mostly hm_0 at the large size. Not worth keeping unless aging is moved off the eviction path, for example by bulk halving.
- Ghost/A1out and TinyLFU gate: full-key ghosts and miss-only, tie-rejecting gates both failed. Any history state must be tiny, cache-friendly, and admit gently.
