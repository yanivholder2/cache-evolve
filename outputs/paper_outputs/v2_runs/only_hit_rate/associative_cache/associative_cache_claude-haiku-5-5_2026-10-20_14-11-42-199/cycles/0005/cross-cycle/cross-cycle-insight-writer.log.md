# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:31:34+00:00

## timeline

### 14:31:42 thinking



### 14:31:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting entries with proven reuse from one-shot inserts is the one direction with repeated hit-rate gains. Probationary-first eviction (Cycle 0003), the shared-frequency admission gate (Cycle 0004), and eviction-next miss insertion (Cycle 0005) all target this problem.\n- A single frequency counter shared by incoming and resident keys is the most consistent positive result. It fixes the scale mismatch that made the earlier admission gate look like a failure (Cycles 0002 and 0004).\n- Gains so far are recorded only qualitatively and on workload-a alone. Treat them as promising until a second workload confirms them.\n\n## Recurring failures\n\n- Hit-rate-only bucket geometry changes (repartitioning, wider or fixed-width buckets, fingerprint filtering) have stayed flat or marginal across Cycles 0001, 0002, and 0005. Their cost benefits were never measured.\n- Aging-based LFU has produced no hit-rate gain in every variant that tested it (Cycles 0001 and 0002). Whether the evaluation ever selects or activates it is still unverified, so these are not clean null results.\n- Bounding or sampling the victim scan left hit-rate flat (Cycles 0003 and 0004). Exact-minimum eviction buys little on hit-rate. Its only remaining case is per-op cost, which is unmeasured.\n- A protected second-hit tier has not recovered. It was negative with forced demotion (Cycle 0004) and still slightly below parent with demotion removed (Cycle 0005). The cap or the second-hit signal is the suspect, not the demotion rule alone.\n- Static-dispatch and other refactors that are meant to be hit-rate neutral show no movement. They need latency measurements to justify them.\n\n## Open questions\n\n- Are the admission gate and eviction-next insert position redundant or additive? Each needs testing alone against the parent.\n- Does the shared-frequency gain hold on a second workload?\n- What do the shared counter, the eviction-next flag, and the second region cost in metadata size and per-op time? No variant has measured either dimension yet.\n- Did the Winner labels follow the hit-rate evidence? Cycle 0002 named a Winner with flat hit-rate, and Cycle 0004 named one that conflicts with the only hit-rate gain. The Winner rule may use a metric that is not recorded.\n- Were the overlays active under the harness? Several insights (Cycles 0003, 0004, 0005) could not confirm this, so their flat or negative results are ambiguous.\n- Is the protected-tier failure caused by the cap size or by the second-hit signal?\n\n## Per-policy notes\n\n- Shared-frequency admission gate: strongest hit-rate signal so far. Keep, and check its cost and second-workload behaviour.\n- Probationary-first eviction: Cycle 0003 Winner and the base for later admission and insert-position work.\n- Eviction-next miss insertion: positive on workload-a. Its metadata cost is claimed to be zero but unverified.\n- Aging LFU: repeated flat results and unconfirmed activation. Do not retune its period on hit-rate alone.\n- Bucket geometry (repartition, fixed-width, fingerprint filter): flat or marginal on hit-rate. Only pursue with per-op time and metadata size included.\n- Victim-scan bounding: flat on hit-rate. Keep only as a cost lever once measured.\n- Protected second-hit tier: negative or slightly negative. Retry only with a changed cap or second-hit signal.\n"
}
```

### 14:31:51 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory with the four required sections in order. I built it only from the inlined ledger digest and did not open the previous file.

- **Recurring wins:** Protecting reused entries from one-shot inserts is the only direction with repeated hit-rate gains. The shared-frequency counter is the most consistent positive result. All gains are qualitative and come from workload-a alone.
- **Recurring failures:** Hit-rate-only bucket geometry changes have stayed flat or marginal. Aging LFU has shown no gain, and whether the harness ever activates it is unverified. Bounded victim scans and the protected second-hit tier also did not pay off.
- **Open questions:** Whether the admission gate and eviction-next insert stack, whether the shared-frequency gain holds on a second workload, and what the new mechanisms cost in metadata and per-op time. Also whether the Winner labels follow the hit-rate evidence, and whether overlays were active.
- **Per-policy notes:** One short group per eviction or insert policy.

Per-op time and metadata size have not been measured in any cycle, so the cost side of every direction is still open.


---

- finished: 2026-10-20T14:31:51+00:00
- duration: 17.34s
- status: ok
- input_tokens: 4
- output_tokens: 3136
- cache_read_tokens: 37479
- cache_write_tokens: 10770
- cost_usd: 0.004097
