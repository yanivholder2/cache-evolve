## Recurring wins

- Leaving strict LRU ordering behind paid off from the start (cycle 1). The hand-based, relink-free hit path that came out of that (CLOCK → SIEVE) has kept get cheap in every cycle since.
- SIEVE has been the strongest policy line since cycle 2. It demotes new keys quickly and promotes reused keys lazily, using one queue and one visited bit. It matched or beat the multi-segment SLRU on hit-rate.
- Changes that alter only the representation and keep eviction order bit-for-bit have never regressed (cycles 3–4). Examples are a singly-linked queue with a predecessor-carrying hand, `u32` links/indices, a packed side array, and padding reuse for policy bits. This is the most reliable way to improve metadata.
- Node compaction helped speed only while hits still relinked (cycle 1). On SIEVE it is purely a memory lever.
- The latency lever turned out to be hashing and map work on the miss path (cycle 4: a faster hasher with `u32` map values). That fits cycle 3's finding that put p99 does not come from the hand.

## Recurring failures

- Stacked scan or loop protection does not add up. BIP inside SLRU probation shrank the second-touch window and cost hit-rate (cycle 2).
- Changing how the hand treats queued nodes keeps costing hit-rate. Deeper per-slot credit (GCLOCK, 2-level SIEVE counter) also made the put tail worse, and capping the sweep evicted real reuse without lowering put p99 (cycles 2–3). SIEVE's single bit looks like a local optimum for reuse signals kept inside the queue.
- A ghost/history signal grafted onto a single queue lost hit-rate and put latency (cycle 4). With no probationary queue to bypass, it only kept long-reuse-distance keys at the expense of better residents, and it added hashing on every miss.
- Explaining the put tail by sweep length was wrong. It was diagnosed in cycle 2 and refuted in cycle 3.

## Open questions

- Does a full S3-FIFO-style split (small probationary FIFO, then main SIEVE, plus a ghost queue) gain the hit-rate that the single-queue ghost graft lost?
- How much put p99 is left after cutting probes per miss: entry-style remove/insert, one hash reused across the probe, no key clone?
- Do the cheaper hasher and the packed side array stack cleanly, given the extra memory touch per hit?
- Every result so far comes from the LIRS workload-a trace at a single cache size (2048). The rankings, and SIEVE's lead in particular, have been unverified for four cycles. This is now the largest overfitting risk.
- The metadata metric under-counts heap tables and side arrays. Per-entry side-array bytes need to be included for comparisons to stay fair.

## Per-policy notes

- **LRU (baseline):** further tweaks to its order are not worth pursuing (dropped in cycle 1).
- **SLRU:** improved hit-rate and nodes over the baseline. Head insertion into probation hurt it. If revisited, tune probation size or admission instead of insertion position.
- **BIP:** improved on its own against loops, with flat metadata. Inside SLRU it worked against the admission window.
- **CLOCK / GCLOCK:** CLOCK gained on all axes in cycle 1. The 2-bit GCLOCK traded put-tail latency for hit-rate, and further single-bit CLOCK tweaks were dropped.
- **SIEVE:** the current lineage. Singly-linked and Fx-hashed SIEVE (cycle 4 v002) is the parent. Changes to counter depth, sweep caps and single-queue ghosts are exhausted. The next steps are miss-path map work, the side-array combination, and an S3-FIFO-style probationary/ghost structure.
