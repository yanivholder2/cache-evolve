# Cross-cycle insights

## Recurring wins

- Policy-neutral representation changes shrink nodes reliably: packing spare bits, narrowing arena links, and sentinel-indexed links all cut size without costing hit-rate. They have not moved `get` much, so credit them for size, not speed.
- Cheap hashing on the key-map and admission paths lowers per-operation cost: a cheaper shared hash for the sketch, and a zero-state map hasher, both helped without changing policy.
- Removing sketch hashing from the read path is a clean read-cost reduction, with only a small hit-rate price on the one trace tested.
- Avoiding list surgery on shared paths tends to pay: the flag-only demotion retry won on per-operation cost, and deferring recency work to eviction time helped `put` more than `get`.
- A bounded second-chance scan gives a predictable miss-path tail, the most consistent per-operation gain in the cohort.
- Reuse-aware and admission-gated eviction is the direction that consistently raises hit-rate on the LIRS workload-a trace.

## Recurring failures

- Per-access work on the hit path rarely pays. Admission hashing on every access, pointer work in the protected rescue, and extra per-lookup mixing all raised `get`/`put` cost without a matching hit-rate gain.
- Ghost-style recently-evicted tracking has weakened. It gained scan resistance, but its miss-path scan and hashing added cost, and promoting ghost hits straight to protected lost hit-rate.
- Hoisted or masked arena index accessors slowed `get` and `put` in more than one cycle. Changes to the shared index helpers are the most reliable source of regressions.
- Growing per-node or per-cache metadata without a clear hit-rate gain has not paid off.
- Shrinking the protected segment cost hit-rate on the scan-heavy trace.
- Removing the list rotation on hit weakened protection. A retained flag or credit recovered only a small part of it; the head has to actually move.

## Open questions

- Do the hit-rate gains of admission gating, ghost tracking, and protected-share tuning hold on workloads without one-hit scans, at other capacities, and on read-heavy traces?
- Is the `get` slowdown after index-width or accessor changes caused by codegen or layout? A codegen-only comparison has not yet been run.
- Are the `put` tail movements and single-run `get` deltas real, or run-to-run noise? Most have not been re-measured.
- Does the zero-state hasher keep its spread and speed on string keys and non-sequential integer keys?
- Can protection come from a recency signal other than list position, without list surgery on the shared path?
- Several Winner labels rest on size alone or on unverified attributions. Do the fitness vectors support a per-operation gain for each?

## Per-policy notes

- Segmented (probationary/protected): the protected share appears to do real work; shrinking it hurt hit-rate. Delayed-demotion rescues recover some hit-rate but add pointer work. A flag-only retry is the cheaper variant, but it gives back some protection.
- Second-chance / referenced-bit: bounded scans give a predictable miss tail. Deferring recency work helps `put` more than `get`. A two-bit credit recovers only a small share of lost protection.
- Admission sketch (frequency-filtered): the largest hit-rate gain seen, from rejecting one-hit keys at capacity. Its cost is per-access hashing, which is now cheaper but still an overhead. Read-path removal lowers cost with a small hit-rate loss; it has not been tested on read-heavy traces.
- Ghost / recently-evicted ring: scan resistance, but at a miss-path cost and with heap storage that struct-size metrics miss. Keep ghost hits probationary if this is retried, and measure the ghost's own hit rate before changing admission.
