# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:11:46+00:00

## timeline

### 20:12:07 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Sketch accuracy is the only lever that has raised hit rate so far. Conservative update (c3), wider `u16` counters at the same byte size (c4), and counting each access once (c5) each added a small gain. Each gain has been smaller than the one before, so this lever is nearly used up.\n- Cutting per-op cost from hashing and layout pays off when it adds no work to the hot path. Fast hashing gave large latency gains (c1), and compact `u32` links halved per-entry `Node` size (c2). Both left behaviour unchanged.\n- Packing metadata into spare bits costs nothing per entry: `u32` links in c2, a region flag in a link bit in c5. Per-node bytes are no longer what blocks structural changes.\n- Changes that are behaviourally neutral merge cleanly. Orthogonal sketch changes have combined with hit rate identical bit for bit (c5).\n\n## Recurring failures\n\n- Sketch aging or decay lowered hit rate every time it was tested. It failed with a 4-bit cap (c2) and with full-width counters plus conservative update (c4). Workload-a rewards stable long-term frequency, and halving removes the margin that keeps scan traffic out of the strict duel.\n- Recency-oriented structure keeps failing to pay for itself. W-TinyLFU + SLRU (c1) and a ~1% LRU window (c5) both lowered hit rate or left it flat, and both added per-hit relink, list-select, and re-hash cost.\n- Tweaking the inputs to the admission duel has not helped. Letting newcomers win ties hurt (c3). LRU-end victim sampling did nothing, because those entries' estimates are nearly equal (c2). The access-accounting fix only changed close duels (c5).\n- Anything that adds correlated collisions hurts admission. The blocked or cache-line sketch broke row independence (c3). The sketch already fits in cache, so cache-oriented sketch layouts have nothing to win.\n- Sketch micro-optimisation gives no gain that lasts. The fused-increment speedup (c4) did not carry over to the `u16` sketch (c5).\n- Latency readings are noisy. In nearly every cycle, some variant got faster or slower with no cause in the code. Treat any latency change without a code-level cause as drift, and only trust side-by-side runs against the parent.\n\n## Open questions\n\n- Is workload-a's hit rate close to its ceiling for a TinyLFU-family design? If so, per-op efficiency is the only axis left.\n- Could a much larger window (about 5–20%) that caches each node's hash or estimate, so the duel does no re-hashing, succeed where the ~1% window failed? Or is any window wrong for this trace?\n- Does per-op cost now sit outside the sketch, in `HashMap` lookup, `key_hash`, and relinking on every hit? This hasn't been tested yet.\n- Is the unkeyed fast hasher safe on adversarial or collision-prone keys? This has been open since c1.\n- Is a milder aging scheme worth trying, such as a very long period or counting only misses? Or should aging be retired for this workload?\n- Several latency regressions were never re-measured side by side against their parent (c2/v002, c5/v002). Were any of them real?\n\n## Per-policy notes\n\n- TinyLFU sketch (CountMin): conservative update with saturating `u16` counters at twice the width is the best setup so far. Keep rows independent. Count each access once. Drop decay.\n- Admission duel: the strict `>` rule with ties going to the victim is correct for workload-a. The duel's inputs now look about as fair as they can get, so further gains need a different structure, not tweaks to the duel.\n- LRU victim/resident list: plain LRU with compact links is the current base. SLRU and windowed variants have lost twice. Any future structure must keep the per-hit cost close to plain LRU.\n- W-TinyLFU window: tested only at ~1%, and it lost there. Larger sizes are untested.\n"
}
```

### 20:12:16 thinking



### 20:12:16 chat

I've written `cross-cycle-insights.md` from the cycles 0001–0005 ledger, using the four fixed sections in order.

The main patterns across cycles:

- **What's worked:** making the sketch more accurate is the only thing that has raised hit rate. Each gain has been smaller than the last. Those changes were conservative update, wider `u16` counters at the same byte size, and counting each access once. Cheaper hashing and compact `u32` links also paid off, cutting cost without changing behaviour.
- **What keeps failing:**
  - Sketch aging hurt hit rate both times it was tried.
  - Recency structures (window + SLRU, a small ~1% window) didn't pay for their extra cost.
  - Changing the inputs to the admission duel didn't help.
  - A sketch layout that breaks row independence made estimates worse.
- **Latency readings:** they drifted with no cause in the code in most cycles. The file says to trust only side-by-side runs against the parent.
- **Open questions:**
  - Is workload-a's hit rate near its ceiling for this design family?
  - Would a larger window (about 5–20%) that caches each node's hash do better than the ~1% one?
  - Is the remaining per-op cost in map lookup, hashing and relinking on every hit, rather than in the sketch?
  - Is the unkeyed fast hasher safe on adversarial keys?
  - Were the latency regressions that were never re-measured real?

The per-policy notes recommend keeping plain LRU with compact links and the strict `>` admission rule as the base. They also suggest testing only much larger windows from here on.


---

- finished: 2026-10-20T20:12:16+00:00
- duration: 30.67s
- status: ok
- input_tokens: 4
- output_tokens: 2018
- cache_read_tokens: 37369
- cache_write_tokens: 10177
- cost_usd: 0.129852
