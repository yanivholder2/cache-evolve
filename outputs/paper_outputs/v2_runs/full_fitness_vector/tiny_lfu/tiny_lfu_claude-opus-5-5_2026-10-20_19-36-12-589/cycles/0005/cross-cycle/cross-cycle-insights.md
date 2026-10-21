## Recurring wins

- Sketch accuracy is the only lever that has raised hit rate so far. Conservative update (c3), wider `u16` counters at the same byte size (c4), and counting each access once (c5) each added a small gain. Each gain has been smaller than the one before, so this lever is nearly used up.
- Cutting per-op cost from hashing and layout pays off when it adds no work to the hot path. Fast hashing gave large latency gains (c1), and compact `u32` links halved per-entry `Node` size (c2). Both left behaviour unchanged.
- Packing metadata into spare bits costs nothing per entry: `u32` links in c2, a region flag in a link bit in c5. Per-node bytes are no longer what blocks structural changes.
- Changes that are behaviourally neutral merge cleanly. Orthogonal sketch changes have combined with hit rate identical bit for bit (c5).

## Recurring failures

- Sketch aging or decay lowered hit rate every time it was tested. It failed with a 4-bit cap (c2) and with full-width counters plus conservative update (c4). Workload-a rewards stable long-term frequency, and halving removes the margin that keeps scan traffic out of the strict duel.
- Recency-oriented structure keeps failing to pay for itself. W-TinyLFU + SLRU (c1) and a ~1% LRU window (c5) both lowered hit rate or left it flat, and both added per-hit relink, list-select, and re-hash cost.
- Tweaking the inputs to the admission duel has not helped. Letting newcomers win ties hurt (c3). LRU-end victim sampling did nothing, because those entries' estimates are nearly equal (c2). The access-accounting fix only changed close duels (c5).
- Anything that adds correlated collisions hurts admission. The blocked or cache-line sketch broke row independence (c3). The sketch already fits in cache, so cache-oriented sketch layouts have nothing to win.
- Sketch micro-optimisation gives no gain that lasts. The fused-increment speedup (c4) did not carry over to the `u16` sketch (c5).
- Latency readings are noisy. In nearly every cycle, some variant got faster or slower with no cause in the code. Treat any latency change without a code-level cause as drift, and only trust side-by-side runs against the parent.

## Open questions

- Is workload-a's hit rate close to its ceiling for a TinyLFU-family design? If so, per-op efficiency is the only axis left.
- Could a much larger window (about 5–20%) that caches each node's hash or estimate, so the duel does no re-hashing, succeed where the ~1% window failed? Or is any window wrong for this trace?
- Does per-op cost now sit outside the sketch, in `HashMap` lookup, `key_hash`, and relinking on every hit? This hasn't been tested yet.
- Is the unkeyed fast hasher safe on adversarial or collision-prone keys? This has been open since c1.
- Is a milder aging scheme worth trying, such as a very long period or counting only misses? Or should aging be retired for this workload?
- Several latency regressions were never re-measured side by side against their parent (c2/v002, c5/v002). Were any of them real?

## Per-policy notes

- TinyLFU sketch (CountMin): conservative update with saturating `u16` counters at twice the width is the best setup so far. Keep rows independent. Count each access once. Drop decay.
- Admission duel: the strict `>` rule with ties going to the victim is correct for workload-a. The duel's inputs now look about as fair as they can get, so further gains need a different structure, not tweaks to the duel.
- LRU victim/resident list: plain LRU with compact links is the current base. SLRU and windowed variants have lost twice. Any future structure must keep the per-hit cost close to plain LRU.
- W-TinyLFU window: tested only at ~1%, and it lost there. Larger sizes are untested.
