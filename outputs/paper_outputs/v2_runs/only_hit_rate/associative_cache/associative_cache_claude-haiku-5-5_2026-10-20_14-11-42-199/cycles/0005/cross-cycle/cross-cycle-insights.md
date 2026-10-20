## Recurring wins

- Protecting entries with proven reuse from one-shot inserts is the one direction with repeated hit-rate gains. Probationary-first eviction (Cycle 0003), the shared-frequency admission gate (Cycle 0004), and eviction-next miss insertion (Cycle 0005) all target this problem.
- A single frequency counter shared by incoming and resident keys is the most consistent positive result. It fixes the scale mismatch that made the earlier admission gate look like a failure (Cycles 0002 and 0004).
- Gains so far are recorded only qualitatively and on workload-a alone. Treat them as promising until a second workload confirms them.

## Recurring failures

- Hit-rate-only bucket geometry changes (repartitioning, wider or fixed-width buckets, fingerprint filtering) have stayed flat or marginal across Cycles 0001, 0002, and 0005. Their cost benefits were never measured.
- Aging-based LFU has produced no hit-rate gain in every variant that tested it (Cycles 0001 and 0002). Whether the evaluation ever selects or activates it is still unverified, so these are not clean null results.
- Bounding or sampling the victim scan left hit-rate flat (Cycles 0003 and 0004). Exact-minimum eviction buys little on hit-rate. Its only remaining case is per-op cost, which is unmeasured.
- A protected second-hit tier has not recovered. It was negative with forced demotion (Cycle 0004) and still slightly below parent with demotion removed (Cycle 0005). The cap or the second-hit signal is the suspect, not the demotion rule alone.
- Static-dispatch and other refactors that are meant to be hit-rate neutral show no movement. They need latency measurements to justify them.

## Open questions

- Are the admission gate and eviction-next insert position redundant or additive? Each needs testing alone against the parent.
- Does the shared-frequency gain hold on a second workload?
- What do the shared counter, the eviction-next flag, and the second region cost in metadata size and per-op time? No variant has measured either dimension yet.
- Did the Winner labels follow the hit-rate evidence? Cycle 0002 named a Winner with flat hit-rate, and Cycle 0004 named one that conflicts with the only hit-rate gain. The Winner rule may use a metric that is not recorded.
- Were the overlays active under the harness? Several insights (Cycles 0003, 0004, 0005) could not confirm this, so their flat or negative results are ambiguous.
- Is the protected-tier failure caused by the cap size or by the second-hit signal?

## Per-policy notes

- Shared-frequency admission gate: strongest hit-rate signal so far. Keep, and check its cost and second-workload behaviour.
- Probationary-first eviction: Cycle 0003 Winner and the base for later admission and insert-position work.
- Eviction-next miss insertion: positive on workload-a. Its metadata cost is claimed to be zero but unverified.
- Aging LFU: repeated flat results and unconfirmed activation. Do not retune its period on hit-rate alone.
- Bucket geometry (repartition, fixed-width, fingerprint filter): flat or marginal on hit-rate. Only pursue with per-op time and metadata size included.
- Victim-scan bounding: flat on hit-rate. Keep only as a cost lever once measured.
- Protected second-hit tier: negative or slightly negative. Retry only with a changed cap or second-hit signal.
