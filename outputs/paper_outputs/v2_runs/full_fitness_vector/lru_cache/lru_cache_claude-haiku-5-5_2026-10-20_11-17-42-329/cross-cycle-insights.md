## Recurring wins

- Cheaper hashing on hot paths keeps paying off. Swapping the admission sketch hash and the key-map hasher lowered per-operation cost with no policy change, while extra finish-step mixing added cost and gave nothing back.
- Node and link narrowing is reliable. Packing flags into spare bits, sentinel indices, and `u32` links shrank nodes without hit-rate loss. Speed has not followed, so treat these as size wins.
- Admission filtering is the lever that has actually raised hit rate. Gating keys at capacity, first by a sketch and then by a doorkeeper on first sightings, kept one-hit traffic from displacing residents. Moving the hash off the read path made the gate cheaper.
- Keeping miss-path and demotion work bounded helps tails. A capped second-chance scan and a flag-only retry in demotion both reduced per-operation cost, though each gave up a little hit rate.
- Consolidating masked index reads into a single helper recovered some cost on the packed layout.

## Recurring failures

- Extra work on the hit path has not paid. Relinks on hit did not move `get`, and per-access hashing added cost to both `get` and `put`.
- Ghost-driven admission has lost hit rate in both directions (to protected and to probation) on the trace tested, and its per-miss scan adds cost to `put`.
- Plain CLOCK, and CLOCK with a bounded sweep, lose the protection or reuse window the segmented policy relies on.
- Placing new keys at the eviction end of probation removes the reuse window and costs hit rate.
- Shrinking the protected share costs hit rate. Growing it helps hit rate but costs `put` tails.
- Hoisted or re-structured arena accessors and the single-lookup `put` rewrite did not speed up `get` or `put`.
- Guarding the reference-bit write cost more than the store it skipped.
- A mask on every index read adds latency, so packed flags only pay off if the read path stays clean.

## Open questions

- Do the admission gate's hit-rate gains hold on other workloads and capacities, and read-heavy traces in particular? Its aging interval, table size, and table footprint are untested.
- Does the protected-share effect hold beyond one trace and capacity, and can a larger protected share be had without the `put` tail cost?
- Can protection be restored without list surgery on the shared path, through a recency signal other than list position?
- Does the zero-state hasher keep its spread and speed on string keys and non-sequential integers?
- Are the recurring `get` and `put` tail and median movements real, or a run-to-run artifact of the measurement setup? Re-measurement has not yet settled this.
- Does the `u32` capacity cap matter for any expected workload?

## Per-policy notes

- Segmented LRU (probation and protected): the protected share is a real hit-rate lever with a `put` cost. Protection depends on the rescued head actually moving, not on a retained flag. A two-bit credit recovered only a small share. The flag-only retry is the cheapest rescue so far.
- CLOCK: the variants tried (plain and bounded sweep) lost protection or hit rate. Per-slot state in a separate allocation adds cost, so CLOCK has not been a winning base.
- Ghost ring: no admission target has held hit rate on the trace tested. The idea is weakened and should only be retried with a measured ghost hit rate and a bounded lookup.
- Admission filter (sketch and doorkeeper): the most promising hit-rate direction. Its cost is hashing on new-key misses at capacity, and read-path work should be kept out of it.
- Policy-neutral layout and hashing changes (node narrowing, map hasher, index helpers) are the safest wins and apply under any eviction policy.
