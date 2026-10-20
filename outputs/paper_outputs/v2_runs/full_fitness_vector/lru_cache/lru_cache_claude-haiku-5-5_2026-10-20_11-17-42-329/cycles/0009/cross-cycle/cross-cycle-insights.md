## Recurring wins

- Cheap hashing on the key map and admission sketch lowers per-operation cost without touching policy. A zero-state map hasher and a single shared sketch hash both helped, while extra finish-step mixing gave no benefit.
- Narrowing arena links and list pointers to `u32` with a sentinel is a policy-neutral way to shrink nodes. Node-size wins have not cost hit-rate, though they do not speed up `get`.
- Keeping the rescued protected head in place, rather than rotating it on the protected-side demotion path, removes per-operation list surgery. The price is a small hit-rate give-back.
- Bounding the second-chance scan trades a small hit-rate loss for a more predictable miss-path tail.
- Removing a hash from the read path cleanly lowers read cost. This is the admission sketch change, which gave up a small amount of hit-rate.
- Consolidating masked index reads into one inlined helper recovers cost on a packed-flag layout.

## Recurring failures

- Ghost-ring admission has not held. Promoting ghost hits to protected lost hit-rate, and promoting them to probation also lost hit-rate on workload-a at 2048. The ghost's per-miss scan and hashing added miss-path cost either way. Treat it as dropped.
- Extra pointer work or list surgery on shared hit and overwrite paths has repeatedly shown up as per-operation cost.
- Changes to arena indexing, including hoisted accessors, narrowed casts, and masked reads, have repeatedly left `get` or `put` slower. Each such change has been an inference rather than a confirmed cause.
- Replacing the segmented structure with CLOCK removed the protection the parent relied on and added per-slot allocation cost.
- Guarding the reference-bit write, and collapsing the `put` miss path onto a single key lookup, did not reduce cost.
- Shrinking the protected share lowered hit-rate, so reductions there are unlikely to pay.

## Open questions

- Are the recurring `get` slowdowns from index access real, or are they codegen or layout effects? A codegen-only comparison has not been run.
- Do the hasher swap, the sketch hash change, and the protected-share effects hold beyond workload-a at 2048? Every result so far is from that one trace and capacity.
- Is the `put` tail movement seen across many variants real, or run-to-run noise? It has never been separated from noise.
- Can protection be kept without list surgery on the shared path, for example through a credit or a different recency signal?
- Several Winner labels do not match the Helped sections of their insights. Fitness vectors should be checked before crediting those variants.

## Per-policy notes

- **Segmented (SLRU) eviction:** The protected share is the main hit-rate lever. Growing it helped on workload-a at 2048 but thinned probation and added `put` miss-path work. Shrinking it hurt. The current range looks like a trade-off. Protection depends on the rescued head actually moving, so a retained flag or two-bit credit recovers only a small share of it.
- **Second-chance and CLOCK:** A bounded second-chance scan is the clearest per-operation gain in this family. A plain CLOCK ring of counters lost the segmented protection, so it is not a substitute.
- **Admission sketch (frequency-filtered):** Rejecting one-hit keys at capacity gave the cohort's clearest hit-rate gain. Its per-access hashing was the cost, and a cheaper hash or a read-path removal reduced it. The read-path removal is the open trade-off, since read-only popularity no longer counts.
- **Ghost ring:** No admission target has held hit-rate on the tested trace. Do not pursue it further without a new signal.
