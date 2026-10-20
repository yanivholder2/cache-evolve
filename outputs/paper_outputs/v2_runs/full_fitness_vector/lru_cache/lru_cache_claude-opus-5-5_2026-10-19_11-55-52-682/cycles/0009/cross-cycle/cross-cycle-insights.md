## Recurring wins

- The safest gains come from changing the representation while keeping eviction order bit-for-bit. Examples: `u32` links, singly-linked SIEVE, the Fx hasher, the packed side array and the VISITED bitset. These gave metadata or latency gains in cycles 0003–0008 without ever costing hit-rate.
- Policies that quickly demote new keys and lazily promote reused keys beat strict LRU order. The line went CLOCK/SLRU, then SIEVE, then S3-FIFO. A single visited bit was enough, and deeper counters never helped.
- Ghosts help, but only behind a probationary queue (the S3-FIFO small + main + ghost split). A short, bounded ghost reach works better than a long one, and a ghost hit should be treated as a strong admission signal.
- The latency levers moved over time. Node layout came first, then hashing (Fx), then where the flags live. Flags written on every hit should sit in a tiny structure addressed by index (the bitset), because the hand and the queue pops read them by index.
- A structural policy change can fix the put tail. S3-FIFO's O(1) small-queue evictions did more for put p99 than capping the sweep did.

## Recurring failures

- Stacking scan-protection mechanisms didn't work. BIP inside SLRU probation shrank the admission window. The half ghost combined with main-eviction ghosting interfered rather than stacked, since both add selectivity and that dilutes the table.
- Deeper reuse credit in the queue (GCLOCK 2-bit, SIEVE 2-level counter) and sweep caps both cost hit-rate. The deeper credit also hurt the put tail.
- Tuning S3-FIFO knobs on workload-a has plateaued for four or more cycles. A wider small queue, a larger or associative ghost, and placing returners at the hand were neutral or negative.
- Hand-rolled key-less hash tables and values stored in map buckets were slower. Wider or unfiltered buckets make every probe, insert and remove more expensive.
- VISITED stored in the map bucket sped up get but turned each hand step into a hashed probe, and the put tail regressed badly.
- The TinyLFU duel run as a speculative hand pass wore down main's VISITED protection and lost on every axis.
- Every result has been measured on one trace (workload-a) at one size (2048). Each cycle has deferred this check, and it is the main threat to every conclusion.

## Open questions

- Does the S3-FIFO (or SIEVE) lead hold at another cache size or trace? This has been unverified for eight cycles and should come before any further tuning.
- Would frequency admission work if it only peeked at the victim without destroying state and counted every main hit? Or does workload-a simply not reward admission signals beyond the ghost?
- Can map buckets shrink below key+`u32` (fingerprint or open addressing into the arena) while keeping cheap tag filtering? The map probe is now the floor for get.
- Can the put-tail map work be cut to a single probe with hashbrown's entry or raw APIs?
- Metadata dimensions still ignore heap side structures (ghost, sketch, side arrays, map growth), so per-node wins are overstated.
- Run-to-run latency drift is about as large as the effects being measured. Parent and variant need to be compared in the same run, with repeated runs.

## Per-policy notes

- **LRU (baseline):** strict recency order lost to every alternative tried on workload-a. Further tweaks to the order it keeps are dropped.
- **SLRU:** an early winner thanks to segmentation and `u32` links, but head insertion into probation hurt. If revisited, adjust probation size or admission, not insertion position.
- **CLOCK / GCLOCK:** the relink-free hit path is cheap. Deeper per-slot credit trades put p99 for hit-rate. This line was superseded by SIEVE.
- **SIEVE:** one queue and one bit came close to a local optimum for signals inside the queue. The singly-linked variant gave free metadata savings. Counter depth and sweep caps are closed.
- **S3-FIFO (current lineage):** the only policy change since SIEVE to raise hit-rate, and it also helped the put tail. Current parent: Fx hasher, VISITED bitset, ghost of main evictions at half size. Ghost size and inputs are closed, the split is near its optimum around 10%, and further gains probably need a different signal or a different trace.
