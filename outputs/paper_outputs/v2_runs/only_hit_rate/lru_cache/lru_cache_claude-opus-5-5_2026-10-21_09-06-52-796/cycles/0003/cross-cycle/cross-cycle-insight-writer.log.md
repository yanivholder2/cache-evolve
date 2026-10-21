# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:23:58+00:00

## timeline

### 09:24:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Strict admission filtering is the only lever that has raised hit rate (cycles 1–3). S3-FIFO's gain over LRU comes from controlling which keys enter `main`, not from how recency is tracked.\n- Tightening the ghost pays off. Cycles 2 and 3 tested it from opposite directions: a longer ghost window lost hit rate and a shorter one (half of capacity) gained. Ghost size is a real tuning knob.\n- The fixed ~10% probation split has held up. Moving away from it, whether adaptively (cycle 2) or by enlarging probation in LIRS (cycle 3), has not helped.\n- Lineage: cycle 1 S3-FIFO → cycle 3 v001 (half-capacity ghost) is the strongest line so far. Cycle 2's variants were dead ends and should not be used as parents.\n\n## Recurring failures\n\n- No cycle has measured `metadata-size` or `per-op-time`; all three reported hit rate only. Every memory or speed claim is untested: CLOCK, the hasher and narrow-index control, the fingerprint ring, the smaller ghost, and LIRS's overhead. This harness gap is the main blocker.\n- Anything that admits keys into `main` without proven reuse while resident has cost hit rate: longer ghost windows, ARC-style adaptive sizing, and fast paths from ghost to `main`.\n- A ghost hit is a misleading reuse signal on loops longer than the cache. Acting on it, whether to resize or to lengthen the window, has hurt.\n- Tuning a single knob alone fails when another path bypasses it. A stricter promotion bar is undone by the ghost-hit route straight into `main`.\n- Planned representation-only experiments (fingerprint ring at the original ghost size, the hasher and `u32`-index graft) have been carried forward twice and never run.\n\n## Open questions\n\n- Where is the optimal ghost size below half of capacity, for example a quarter? At what point does it start dropping genuine re-references?\n- Would sending ghost hits into `small` instead of `main` beat simply shrinking the ghost? Would that finally make `PROMOTE_FREQ` > 1 meaningful?\n- Once measurement exists: does the S3-FIFO ghost cost more memory per key than removing the links saves? Does CLOCK's sweep cause tail-latency spikes on `put`?\n- Are the representation changes (Fx hasher, `u32` links, MRU fast path) independent of policy, so they can be stacked onto the best S3-FIFO variant for a free speed gain?\n- What actually explains the hit-rate gap between S3-FIFO and LIRS? Non-resident re-admission handling is the leading suspect, since probation size has been ruled out.\n\n## Per-policy notes\n\n- **S3-FIFO:** the most successful policy throughout. Keep the ~10% probation split. Ghost size matters a lot; smaller has been better so far. The ghost path and the promotion bar interact, so tune them together. Its ghost structures sit outside `Node` and their memory cost has never been measured.\n- **CLOCK:** matched LRU hit rate, which shows exact LRU ordering is not needed. It has no hit-rate upside. Revisit only if measurement shows a memory or latency win.\n- **LIRS:** beats LRU but trails S3-FIFO. It is heavier: larger `Node` and a linear walk to enforce the ghost bound. Enlarging the HIR fraction regressed. Set it aside unless costs can be measured and re-admission is reworked.\n- **ARC-style adaptive S3-FIFO:** drifted from the good split and admitted on ghost hits, which lost hit rate. Abandoned.\n- **Strict LRU + representation changes (control):** hit rate unchanged by design. Its value is purely speed and memory, which is still unmeasured and a candidate to graft onto the best policy.\n"
}
```

### 09:24:19 chat

I wrote `cross-cycle-insights.md` in the output dir, built only from the cycle 1–3 ledger. It has the four required sections in order.

- **Biggest gap:** none of the three cycles measured `metadata-size` or `per-op-time`; every cycle reported hit rate only. So every memory or speed claim so far is unverified, and the representation-only experiments planned twice were never run.
- **What works:** strict admission filtering is the only thing that has raised hit rate. Ghost size was confirmed as a real tuning knob: a longer ghost lost hit rate in cycle 2, and halving it gained in cycle 3. The fixed ~10% probation split has held up.
- **What fails:** any path that lets keys into `main` without proven reuse has cost hit rate. That covers longer ghost windows, ARC-style adaptive sizing, and fast paths from a ghost hit straight to `main`. A stricter promotion bar alone also fails, because ghost hits bypass it.
- **Policies:** S3-FIFO leads, from cycle 1's version through cycle 3's half-capacity ghost; cycle 2's variants are marked as dead ends. CLOCK only matches LRU, LIRS trails S3-FIFO at higher cost, and the ARC-style variant is abandoned. The strict-LRU control is kept only as a speed graft to try once measurement exists.
- **Open threads:** a ghost smaller than half of capacity, sending ghost hits into `small` instead of `main`, and grafting the hasher and narrow-index changes onto S3-FIFO.


---

- finished: 2026-10-21T09:24:20+00:00
- duration: 21.47s
- status: ok
- input_tokens: 4
- output_tokens: 1968
- cache_read_tokens: 35711
- cache_write_tokens: 8487
- cost_usd: 0.114414
