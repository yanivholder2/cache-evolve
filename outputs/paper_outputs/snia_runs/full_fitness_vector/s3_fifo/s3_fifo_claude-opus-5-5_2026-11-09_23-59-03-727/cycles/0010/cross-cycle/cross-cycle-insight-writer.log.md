# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-11-10T04:52:12+00:00

## timeline

### 04:52:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **One change per variant.** Cycles 1–2 bundled changes, and every result was just a trade between hm_0 and web_0. Since cycle 3 each variant changes one thing, and that is the only reason mechanisms got pinned down and real gains became visible.\n- **Filters that give credit only for reuse inside main.** Ghosting only main victims that were hit while resident (c3) and resetting freq on graduation (c3) each helped. Combined in c4, they gave the largest gain in the run, at every trace/size point. Selective ghost use beats bigger ghost budgets.\n- **Small-share changes gated on cache size.** Changing the small share only at capacities up to 32768 cannot hurt the saturated 131072 cells. 1/4 → 1/5 (c9) → 1/6 (c10) each gave a small gain at 16384. These are the only hit-rate wins since c5, and each step gains less than the last.\n- **Structural speedups that keep hit rate bit-identical.** Slab plus hash indexes (c2) and the Fx hasher (c3) are the only timing gains that the diffs actually explain. Refactors that preserve hit rate reproduce the parent exactly, so implementation-only variants can be trusted not to change behavior.\n- **MAX_FREQ 7 with decrement-by-one aging, and freq reset to 0 at graduation, is a local optimum.** Changes in either direction have hurt.\n\n## Recurring failures\n\n- **Resizing the ghost or reshaping its container.** A ghost larger than `main_size` (c4) let stale reuse back into main. A ring (c7), fingerprints (c8) and splitting the ghost into regions (c2) all failed to help hit rate or made the `put` tail worse.\n- **Changing ghost→main admission in either direction.** Looser admission (c6) and stricter admission (c7) both regressed web_0 at 131072.\n- **Changing probation length or the global small share.** A global 10% (c5) and a global 1/3 (c6) both lost. So did extra reinsertions inside small (c6), which mostly duplicated the ghost path and pushed out newcomers.\n- **Tenure or credit that keys have not earned in main.** Halving freq on each sweep (c7), MAX_FREQ 15 (c8), carry-over credit at graduation (c2, c8) and `freq >= 1` graduation (c2) all hurt hm_0 at 16384.\n- **Timing changes that add work to the common path to remove rare work.** Per-insert liveness probes (c7), per-miss stale-tail trims (c10), `entry()` with its hidden `reserve` (c9), over-allocated indexes (c5), a merged index (c4), a CLOCK ring (c9), sweep caps (c10) and an unbounded multi-pop eviction loop (c6) all made the `put` tail worse.\n- **Timing hypotheses inferred from code instead of measured.** \"Probe count drives p99\", \"footprint drives p99\" and \"sweep length drives p99\" have each been disproved in turn.\n- **Per-op-time shifts on policy-only diffs.** These have been noise or environment drift in every cycle and deserve no credit.\n\n## Open questions\n\n- Where between 1/6 and 1/10 does the small share at 16384 peak, and does web_0 lose first? One final probe (1/7 or 1/8), then treat hit rate as converged.\n- What actually sets the parent's `put` p99: ghost insert and compaction, rehash, sweep steps, or noise? It needs profiling before any more timing code changes.\n- Is web_0's gap at 131072 out of reach for this policy family? Every structural lever tried has failed on it.\n- Would a fingerprint ghost with a pass-through hasher, giving one hash per miss, win on `put`? It is the only probe-reduction idea left untested.\n- The metadata-size metric has stayed flat throughout and misses ghost and index heap growth. Should it also track heap bytes per resident key?\n\n## Per-policy notes\n\n- **S3-FIFO (the only policy explored).** Current parent: Fx slab, reuse-gated ghosting of main victims, freq reset on graduation, MAX_FREQ 7, ghost bound = `main_size`, and a size-gated small share of 1/6 at ≤32768 (1/4 above).\n- **By trace.** hm_0 responds to how main retains keys, and its remaining headroom is at 16384. web_0 depends on the probation window, and its gap at 131072 has resisted every lever.\n- **By size.** 131072 is saturated against changes that leave hot-key retention alone, but it does respond (badly) when hot keys get evicted, as with the c10 sweep cap. Any remaining hit-rate gains are at 16384.\n- **Timing.** Keep eviction work bounded per miss, never add unconditional work to the miss path, and count the per-entry 24-byte budget as already used. The tail is set by heavy rare events and hidden map work, not by bookkeeping per step.\n"
}
```

### 04:52:38 thinking



### 04:52:38 chat

I've written `cross-cycle-insights.md` from the ten-cycle ledger, with the four required sections in order and no numbers beyond the parameter values the ledger itself names.

- **Hit rate is close to done.** The only gains since cycle 5 came from changing the small-queue share at small cache sizes only (1/4 → 1/5 → 1/6), and each step gained less. The file recommends one last try at 1/7 or 1/8, then treating hit rate as converged.
- **Most other levers have failed.** Changing the ghost size or container, loosening or tightening how ghost hits re-enter main, changing probation length, and giving keys extra frequency credit have all hurt, often in both directions.
- **What worked earlier:** testing one change per variant, and the two filters that give a key credit only for reuse while it sits in main. Combined in cycle 4, those filters gave the largest gain in the run.
- **Timing:** the only timing gains the code explains came from the slab-plus-hash rewrite (cycle 2) and the faster hasher (cycle 3). Every timing guess since cycle 7 was inferred from code and turned out wrong. The file says to profile what actually drives the slowest `put` calls before changing more code, and never to add work to every cache miss.
- **Open threads:** whether web_0's gap at the large cache size is out of reach for S3-FIFO, whether ghost fingerprints with a pass-through hasher would speed up `put`, and whether the per-entry metadata metric should also count heap memory, since it has stayed flat while heap use grew.

The per-policy section covers S3-FIFO only, because it is the only policy the ledger explored.


---

- finished: 2026-11-10T04:52:39+00:00
- duration: 26.48s
- status: ok
- input_tokens: 4
- output_tokens: 2378
- cache_read_tokens: 44025
- cache_write_tokens: 15896
- cost_usd: 0.184666
