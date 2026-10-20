## Recurring wins

- Scan-resistant insertion/eviction policy is the only lever that moved hit rate (cycles 0002-0004). Keeping one-hit-wonders from displacing reused entries beat any capacity, geometry or position-only change.
- Cheap, stateless mechanisms (insertion position, swap-based placement) win over stateful ones. Gains rank by how little bookkeeping they add.
- Put-path cost reductions are real and repeatable: removing the half-bucket shift (O(1) swap insertion) and redundant hash/modulo work both cut put latency without hurting hit rate.
- Exact ordering among displaced entries doesn't matter. Where the new entry lands matters.
- Metadata size has never grown in any cycle; the extra state fit existing padding or wasn't needed.

## Recurring failures

- Get-latency gains: five cycles, none real. Arithmetic, hashing, item movement, bucket layout (tags, alignment) and geometry are all ruled out. Small get shifts look like noise or codegen effects.
- Position-only LRU tweaks and geometry sweeps (cycle 0001) were dead ends. Associativity is not the hit-rate limit.
- Stacking scan-resistance mechanisms does not compound (reference bit plus low insertion lost on both axes). Overlapping signals cost more than they add.
- Added per-access logic (frequency counters, hash-conditional branches, parallel tags kept in sync) slows put and get slightly.
- Weak admission signals, such as a one-entry-per-bucket ghost, add put work and slightly hurt hit rate.
- Put-latency gains with no mechanism (cycles 0001, 0002, 0005) keep appearing and are probably noise. Winner labels often rest on this thin evidence, even when the variant's own insight reports a loss.

## Open questions

- Where does get time go, if not hashing, reordering or layout? Candidates are harness overhead, dispatch and the key/value path. This is unmeasured after five cycles.
- Is the optimum insertion point below len/4 (such as len/5 or len/8)? Moving toward the protected end lost hit rate.
- Does plain 64-byte alignment reliably improve put tail latency, and does it stack with swap insertion?
- Which Winner statuses are real versus noise? Several Winners had negative self-reported insights.
- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic? No cycle has tested this.
- Would a deeper or more selective admission history help, or is scan resistance already saturated?

## Per-policy notes

- Baseline LRU with remove+push promotion: hit rate and get are insensitive to geometry, promotion form (rotate) and partial promotion. Partial promotion hurts hit rate.
- CLOCK reference bit: cheap, and it lowers put cost and get tail when used alone. It hurts when stacked on low insertion, because a one-bit second chance is too coarse.
- Frequency counter (2-bit, LRU tie-break): marginal hit gain, per-access latency cost. Dropped.
- LIP/BIP-style low insertion: clearest single hit-rate gain. Inserting at the pure LRU end starves new entries. The sweet spot is len/4 to mid-bucket, with explicit transpose promotion on hit.
- Mid/len/4 insertion with transpose, using swap-based O(1) placement: current policy baseline. Moving toward len/3 lost hit rate.
- Ghost admission filter (single entry): net negative. It is not worth pursuing in that form.
