# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T14:48:28+00:00

## timeline

### 14:48:47 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **Admission filtering beats recency tracking.** S3-FIFO (small probation + ghost + `main`) beat LRU in cycle 1. It has stayed the lineage base since then. Changes to how recency is tracked (CLOCK, exact LRU order, SIEVE) never added hit rate.\n- **Shrinking the admission window paid off, with smaller gains each time.** The ghost was halved three times (cycles 3–5), then probation slots were moved into the ghost (cycles 6–8). Every step helped, each less than the last. Both curves are now flat. The split looks settled with `small` between a fortieth and an eightieth of capacity, `small + ghost` held near a fixed total, and the ghost larger than `small`.\n- **`main` works best with slow, forgiving retention.** A higher credit cap (`MAX_FREQ = 7`) gave small gains (cycles 6, 9, 10). Losses came from faster demotion (SIEVE), halving-based aging, and a lower cap. `main`'s full CLOCK rotation already protects zero-credit arrivals that are reused late.\n- **Changes to how the ghost is stored can be kept separate from policy.** The fingerprint ring gave exactly the same hit rate every time it was tried (cycles 5, 9, 10), and it is now in the base. Current parent: ring + cap 7 + `small`-to-`main` carryover.\n\n## Recurring failures\n\n- **Ghost and admission were loosened.** A longer ghost and ARC-style adaptive `small` sizing (cycle 2) both let keys into `main` without proof of reuse, and both cost hit rate.\n- **Gates were stacked on top of a short ghost.** Two-hit promotion bars, routing ghost hits through `small`, count-min gates and full TinyLFU (cycles 3–5) gained little or regressed. Once the window is short, a ghost hit is a good reuse signal, and admission can also be too strict. Gates on only one path leak through the other path.\n- **Ghost arrivals were given starting credit.** This lost in cycle 7, yet the per-variant insights proposed it again in cycles 8, 9 and 10. It is closed. Do not retry it.\n- **`main` was made to shed credit faster.** SIEVE, halving-based aging and the 2-bit cap all regressed. Stale credit is not the bottleneck.\n- **New ghost sources and more credit work on the `small` path found nothing.** The `main`-fed ghost was flat. Carryover mostly overlaps with a small `small`.\n- **Memory and speed claims were made without measurement.** For ten cycles only `hit-rate` was scored. Every hypothesis aimed at `metadata-size` or `per-op-time` was left untested. These include CLOCK, the hasher and `u32`-link graft, the fingerprint ring, the TinyLFU sketch and the LIRS node size.\n\n## Open questions\n\n- **Harness gaps (blocking):**\n  - Can `metadata-size` and `per-op-time` be scored? Does the ring actually shrink metadata, and how do S3-FIFO variants compare with the baseline's cost?\n  - Do the split ratio, cap and carryover gains hold at a second cache size or on a second trace? This has been asked for since cycle 4 and never delivered. The last several gains are within noise on workload-a at 2048.\n- **Cap/carryover ablation:** Does cap 7 without carryover match the parent? If so, carryover can be removed to simplify the code.\n- **Hasher and index graft:** Can cycle 1 v003's Fx hasher and `u32` indices, and a single-hash ring lookup, be added as policy-neutral speed changes once speed is measurable?\n- **Where is the remaining headroom?** Every known admission and retention lever is saturated on workload-a at 2048. New levers should only be proposed after the evaluation broadens.\n\n## Per-policy notes\n\n- **LRU (baseline):** This is the reference point. Its exact ordering adds no hit rate over CLOCK on this trace. Its speed and memory have never been measured against the variants.\n- **CLOCK:** Hit rate roughly matches LRU. Its claimed speed and memory wins were never measured, and the risk of slow victim sweeps is unexamined. Parked.\n- **S3-FIFO:** This is the winning lineage.\n  - Hit rate is sensitive to the probation/ghost split, and the split is now tuned.\n  - `main` uses CLOCK reinsertion with cap 7.\n  - The fingerprint-ring ghost is in the base, with sequence-number-correct bookkeeping.\n  - On the current trace and size, its tuning is saturated.\n- **LIRS:** It beat LRU but stayed below S3-FIFO. It is heavier, with a larger node and a linear ghost-bound walk. A larger HIR fraction made it worse. Set aside until cost can be measured and the bound walk is O(1).\n- **ARC-style adaptivity / TinyLFU / SIEVE:** Each was tried as a modification of S3-FIFO and each either regressed or gained only marginally. Bring TinyLFU back only if cost measurement shows it is close to free.\n"
}
```

### 14:48:53 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle folder. It has the four required sections in order and is built only from the ledger digest.

The main conclusions across the ten cycles:

- **Measurement is the real blocker.** Only hit rate has ever been scored. Every claim about `metadata-size` or `per-op-time` is still untested. A second cache size or trace has been requested since cycle 4 and never run.
- **What keeps working:** S3-FIFO's admission filtering has been the winning line since cycle 1. Shrinking the ghost and then moving probation slots into it helped every time, with smaller gains each step, and both have now levelled off. `main` does best when it keeps credit longer rather than dropping it faster.
- **Tuning is used up on the one trace (workload-a at cache size 2048).** Every known admission and retention lever has stopped paying, and recent gains are within noise. The current parent is the fingerprint ring plus cap 7 plus the `small`-to-`main` carryover.
- **One idea has to stay closed.** Giving ghost-admitted keys a starting credit lost in cycle 7, but the per-cycle notes proposed it again in cycles 8, 9 and 10. The file marks it as closed.
- **Per-policy status:** CLOCK and LIRS are set aside. ARC-style adaptivity and SIEVE are recorded as failures. TinyLFU is set aside too, unless cost measurement shows it costs almost nothing.

The only cheap hit-rate check left is cap 7 without carryover. If it matches the parent, the carryover code can be removed.


---

- finished: 2026-10-21T14:48:53+00:00
- duration: 25.56s
- status: ok
- input_tokens: 4
- output_tokens: 2434
- cache_read_tokens: 43290
- cache_write_tokens: 16517
- cost_usd: 0.18949
