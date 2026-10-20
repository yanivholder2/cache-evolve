# Cross-Cycle Insights

## Recurring wins

- **Representation changes that leave eviction order untouched are reliably safe.** Over five cycles they include u32 links, the singly-linked queue, the Fx hasher, packed side arrays and the VISITED bitset. Each one kept hit-rate exactly and usually improved metadata. Only policy changes have caused hit-rate regressions.
- **Moving away from strict LRU recency order pays off.** Second-touch promotion, second-chance bits, quick demotion of new keys and probationary admission all beat LRU on workload-a.
- **Simple single-bit policies with lazy promotion beat heavier multi-segment schemes.** SIEVE beat SLRU, GCLOCK and BIP. S3-FIFO then added the only further hit-rate gain.
- **Ghost history helps only alongside a probationary queue to bypass.** It lost as a graft onto single-queue SIEVE and won inside an S3-FIFO small/main/ghost split.
- **The main latency levers moved over time.** First it was node layout and a hit path with no relinking. Then it was cheap hashing, which mainly helped the miss path and put p99. Now it is how many separate memory locations a hit touches. When hit flags must live outside the node, they belong in a tiny, always-cached structure (bitset).
- **Structural policy changes can fix the put tail.** S3-FIFO's O(1) small-queue evictions improved put p99 in a way that capping the sweep never did.

## Recurring failures

- **Stacking scan defenses.** BIP inside SLRU probation only shrank the window in which new keys can earn a second touch.
- **Deeper in-queue reuse credit.** This includes GCLOCK's 2-bit counter and the SIEVE 2-level counter. Each time it cost hit-rate or put tail, and SIEVE's single bit looks like a local optimum.
- **Capping the hand sweep.** It lost hit-rate and did not move put p99, because long sweeps carry real reuse and are not what sets the tail.
- **Hand-rolled key-less hash tables.** They cannot reject a bucket without touching the arena, so they lose badly to hashbrown at the put tail.
- **Making ghosts more aggressive.** Ghosts that are larger or more associative, wider small queues, and returners placed at the hand all cost hit-rate. Capacity in main for proven-reuse keys is the scarce resource. The ghost's bounded reach works as a useful filter, and a ghost hit should keep full admission.
- **More node-shape work aimed at speed.** It stopped moving latency once the hit path was relink-free. It is now a memory lever only, and side arrays can add get cost on policies with heavy miss paths.
- **Tuning knobs on one trace and one size.** For four cycles, S3-FIFO admission and ghost tweaks have produced only noise-sized moves. The plateau has been reached.

## Open questions

- **Generalization:** every result comes from workload-a at one cache size. Does the SIEVE to S3-FIFO ranking hold elsewhere? This has been flagged unverified in every cycle and is now the top priority.
- **Measurement noise:** latency drift is now comparable to the effects being measured, and changes have slowed paths they never touched. Repeated runs or confidence bands are needed before trusting small latency deltas.
- **Metadata accounting:** the metric still ignores heap side structures (ghost buffer, side arrays, bitset, Vec headers). Count them per entry.
- Should evictions from main also feed the ghost?
- At what ghost size does hit-rate start to fall, and does that point move with cache size?
- Can hashbrown's entry or raw-table API give a single-probe miss path while keeping its tag filtering?
- Can one fused node-plus-flag layout match the bitset's get gain without an extra array header or extra work on each miss?

## Per-policy notes

- **LRU (baseline):** dominated on every dimension by every departure from it. Do not tweak its recency order further.
- **SLRU:** a solid early win from second-touch promotion and u32 links. It does not combine with BIP. If revisited, change the probation size or admission, not the insertion position.
- **CLOCK / GCLOCK:** relink-free hits and a small node. Deeper credit trades put-tail latency for hit-rate, so it is superseded by SIEVE.
- **BIP:** a decent first-cycle win on its own, but harmful inside SLRU.
- **SIEVE:** best single-queue policy. A singly-linked queue and the Fx hasher are pure wins. Counter depth, sweep caps and ghost grafts all lost.
- **S3-FIFO (current lineage):** the only hit-rate gain since SIEVE, plus a better put tail. The best configuration is a small queue of about 10%, a direct-mapped (half-size or smaller) fingerprint ghost with full admission on ghost hits, a VISITED bitset and a key swap on eviction. Its knobs have plateaued on workload-a, and its put path is dominated by hashing and ghost probes on each miss.
