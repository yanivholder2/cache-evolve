## Recurring wins

- Policy-neutral layout changes that shrink nodes or structs have held up: narrowed arena links with a sentinel index, and flags packed into spare link bits with the masking consolidated into one helper. Hit-rate stayed intact in each case.
- Cheaper hashing on hot paths lowers per-operation cost without changing policy: the zero-state key-map hasher and the shared admission-sketch hash both helped. Extra finish-step mixing added cost and gave nothing.
- Deferring recency work to eviction time, and keeping a rescued protected head in place rather than moving it, cut put-path cost.
- Bounding the second-chance eviction scan trades a small hit-rate loss for a predictable miss-path tail.
- Consolidating masked index reads into one helper recovered per-operation cost on the packed layout.

## Recurring failures

- Work added to every access on shared paths keeps costing more than it returns: per-access sketch hashing, finish-step mixing, and the flag mask on every link read.
- List surgery on hit or overwrite paths for recency protection has not paid off. Removing relinks did not move `get`, and restoring protection by moving the head cost hit-rate or per-operation time.
- Ghost-ring admission has lost hit-rate in both directions (promotion to protected, and promotion to probation) on workload-a at 2048.
- Shrinking the protected segment hurt hit-rate on the scan-heavy trace.
- A credit bit in the flags byte recovers only a small part of the protection lost when list rotation is dropped.
- Rewriting the `put` miss path to a single key-map lookup did not reduce `put` cost.
- Index-access rewrites (narrowed indices, hoisted accessors) slowed `get` or `put` without a confirmed cause.

## Open questions

- Do the hit-rate gains from admission gating and the segmented policy hold on other workloads and capacities? Most results come from workload-a at 2048.
- Are the recurring `get` and `put` tail movements real or run-to-run noise? Re-measure before crediting or blaming any mechanism.
- Does the zero-state hasher keep its speedup and bucket spread on string keys and non-sequential integer keys?
- Is the index-width slowdown caused by codegen or accessor structure? A codegen-only comparison has not been run.
- Can protection for rescued heads be restored without list surgery on the shared path?
- Several insights were written from hypothesis text and fitness vectors because diffs could not be inspected, and the v002 node growth in cycle 7 is unexplained. Code-level attributions remain unverified.
- The cycle 1 v003 strategy (reference-aware eviction) was never evaluated because of a dependency fetch failure.

## Per-policy notes

- Segmented (probationary/protected) LRU: the 4/5 protected share appears to be doing real work. Protected-side rescue recovers hit-rate at a per-operation cost. A flag-only retry is cheaper but gives back a little protection.
- Second-chance / referenced-bit: deferred recency and a bounded scan give the clearest put-tail gains. Hit-rate on re-referenced keys drops slightly.
- Frequency-sketch admission: gains hit-rate by rejecting one-hit keys at capacity, but its hashing has been the main cost. Removing the read-path hash lowers read cost and costs a small hit-rate loss, since read-only popularity no longer counts.
- Ghost ring: no promotion target has held hit-rate on workload-a at 2048. Its per-miss scan and extra hashing land in `put`, and its heap storage is not in the struct-size metric.
