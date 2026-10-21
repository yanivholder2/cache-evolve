## Recurring wins

- Cheap scan resistance moves hit rate: policies that stop one-hit-wonders from displacing reused entries (low or mid-position insertion, reference bit alone) paid off from cycle 0002 onward. Capacity and associativity were not the limit.
- Mid-bucket (about len/4) insertion with one-step transpose promotion is stateless and reproducible (cycles 0003, 0006). Swap-based insertion (cycle 0004) cut its put cost with no hit-rate change.
- Put-path cost is the lever that moves: removing shifts, redundant hashing and modulo cut put latency. Exact order among displaced entries doesn't matter; where the new entry lands does.
- Gradual one-step promotion works as a frequency filter on the scan-heavy trace.

## Recurring failures

- Get latency has had no real gain in seven cycles. Item movement, arithmetic, hashing, tag-first layout and alignment are all ruled out. Small get shifts are noise or codegen effects.
- Position-only tuning is saturated. Wider buckets, len/3, len/5 and LRU-end insertion all failed to help, so len/4 is a local optimum.
- Stacking scan-resistance mechanisms (reference bit plus low insertion) does not compound. It loses on both hit rate and speed.
- Per-slot frequency counters and one-entry ghost filters add per-access work for little or negative hit-rate gain.
- Stateless randomization (hash-parity victim, hash-conditional escapes) carries no reuse signal. It adds branches and noise.
- Aggressive promotion (geometric jump) removes the frequency filter and hurts hit rate.
- Put-hit control-flow tweaks are worthless because puts are mostly misses.
- Allocation-only layout tweaks (alignment) did not reproduce their put-tail gain.
- Put-latency gains with no mechanism have repeatedly turned out to be noise.
- Winner labels are unreliable. For five or more cycles, Winners included variants whose own insight reports a loss. Weigh the mechanism and the insight text over the label.

## Open questions

- Which Winners are real? A noise check (repeat runs) is needed.
- Where does get time go? Harness, dispatch, or the key/value path is untested.
- Can a small real reuse signal fit in the 40-byte metadata, for example a jump only after a second hit, or a deeper selective admission history?
- Do any gains hold on a second workload, or are they specific to workload-a's scan-heavy traffic?
- Can the put-miss and insertion path be made cheaper still?

## Per-policy notes

- LRU baseline (move-to-front): position and rotate tweaks gave nothing. Hit-rate gains need a scan-resistance signal.
- CLOCK reference bit: cheap and helpful alone (it removes reordering). It hurts when stacked on low insertion, because a one-bit second chance is too coarse.
- Frequency counter (2-bit, LRU tie-break): marginal hit-rate gain, with latency cost on every hit and put. Dropped.
- Low or mid insertion (LIP/BIP-style): the clearest gain. Too deep (LRU end) starves new entries; len/4 is the sweet spot. Swap-based insertion with transpose promotion is the current baseline.
- Ghost admission filter: one entry per bucket is too weak and slightly hurts hit rate.
- Hash-based eviction and geometric promotion: both lose hit rate. Don't revisit.
