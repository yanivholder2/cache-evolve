# Cycle 0003 Insight

## Cohort outcome

Variants explored the insertion position and promotion granularity: LRU-end insertion with a rare hashed MRU escape (v001), a CLOCK reference bit stacked on low-position insertion (v002), and mid-bucket insertion with transpose-by-one promotion (v003). v001 and v003 were Winners, but the evidence for v001 is weak, since its own insight reports hit rate and latency as slightly worse than the parent. v002 was the only clear loser.

## Patterns

- Stacking scan-resistance mechanisms did not compound. The reference bit combined with low insertion lost both hit rate and speed, which contradicts the cycle 0002 hope that the cheap mechanisms would add up.
- Insertion depth has a sweet spot. Pushing toward the LRU end starves new entries of the chance to prove reuse. A middle or len/4 position keeps scan resistance and still lets reused entries survive.
- Explicit promotion on hit matters when insertion is low. Proven entries have to be moved up, and a one-bit second chance is too coarse to do that.
- Latency again followed added per-access logic (hash-conditional branches, bit masking, mid-bucket shifts). Get time did not improve in any variant, which is further evidence that reordering is not what dominates it.
- No variant changed metadata size.

## Open questions

- Are the Winner status and hit-rate edge of v001 real, or within noise? Its insight describes a marginal loss.
- Can mid-insertion keep its hit-rate gain without the median put cost of shifting half the bucket?
- Where does median get time go (hashing, bucket lookup, dispatch)? This has been unmeasured for three cycles.
- Do any of these gains hold on workloads other than workload-a?

## Continuation

Stop varying insertion position and stacking reference-bit schemes; the position axis is near saturation, with len/4 to mid-bucket the useful range. Treat v003's mid-insertion plus transpose as the policy baseline and try to cut its put cost, for example with a cheaper insertion mechanism that avoids shifting. Direct the next latency hypotheses at the lookup path (hashing, bucket search, dispatch) and validate on a second workload.
