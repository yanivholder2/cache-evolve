## Recurring wins

- Stateless, cheap scan resistance moves hit rate: low or mid-bucket insertion with transpose-by-one promotion. Cycles 0002, 0003 and 0006 reproduced this.
- Cutting put-path cost works. Swap-based O(1) mid-insertion (0004) removed the half-bucket shift, and hash-once indexing also cut put time. Hit rate and metadata were unchanged.
- Exact order among displaced entries doesn't matter. Only where the new entry lands does.
- Explicit promotion on hit is needed once insertion is low.
- Metadata size has never changed in any cycle. Every win so far came without extra state.

## Recurring failures

- Position-only tweaks to LRU and geometry (0001): wider buckets and partial promotion gave no gain.
- Retuning the insertion point (len/3 in 0005, len/5 in 0006) failed on both sides. len/4 is a local optimum, and the axis is exhausted.
- Stacking scan-resistance mechanisms (reference bit plus low insertion) did not compound. It lost hit rate and speed.
- Per-slot frequency counters and one-entry ghost filters add per-access work for little or negative hit-rate gain.
- Get latency has not improved in six cycles. Item movement, arithmetic, hashing, tag-first scans and alignment are all ruled out. Small get shifts are noise or codegen effects.
- Allocation-only layout changes such as 64-byte alignment are not reliable. The put-tail gain in 0005 did not repeat in 0006.
- Latency gains with no mechanistic reason (put latency in 0001, 0002 and 0005) have been noise. Discount them.
- Winner labels are weakly tied to evidence. Since 0003, many Winners have insights reporting a loss. Treat such Winners as suspect.

## Open questions

- Which Winner statuses are real? A noise check and repeat runs are needed.
- Where does get time go if not bucket memory, hashing or reordering? Candidates are harness or dispatch overhead and the key/value path. It has never been measured directly.
- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic?
- Can mid-insertion with transpose avoid its put-tail penalty when combined with swap-based insertion?
- Can the hit-promotion path or a small per-slot signal under the metadata budget give a gain that insertion position can't?
- Would a deeper or more selective admission history help, or is scan resistance already saturated?

## Per-policy notes

- Baseline LRU: hit rate is insensitive to associativity and to partial promotion. Rotate-based moves gave no get speedup.
- CLOCK reference bit: it cheapened put and the get tail on its own (0002). Stacked on low insertion it lost (0003), because a one-bit second chance is too coarse.
- 2-bit frequency counter: small gain, latency cost on every hit and put. Dropped.
- Low-position (LIP/BIP-style) insertion: the clearest single gain (0002). Going all the way to the LRU end starves new entries, so a hashed MRU escape is marginal.
- Mid-bucket or len/4 insertion with transpose promotion: the settled policy baseline. Swap-based insertion is its cheap implementation.
- Ghost admission filter (one entry per bucket): too weak. It slows put and slightly hurts hit rate.
