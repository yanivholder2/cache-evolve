## Recurring wins

- Scan-resistant insertion/eviction policy is the only lever that has moved hit rate (cycles 0002, 0003). Keeping one-hit-wonders from displacing reused entries pays off; capacity and associativity are not the limit.
- Cheap, low-bookkeeping mechanisms rank best: stateless insertion position and the reference bit beat the frequency counter in 0002.
- Intermediate insertion depth (len/4 to mid-bucket) with explicit promotion on hit keeps scan resistance while letting new entries prove reuse. This is the current policy baseline (0003 v003).
- Latency tracks added per-access logic: fewer branches or less reordering means cheaper puts and a better get tail.

## Recurring failures

- Position-only LRU tweaks and geometry sweeps (wider buckets, partial promotion) gave flat or worse hit rate (0001). The cause is that conflict misses and promotion granularity are not the bottleneck.
- Rotate/memmove micro-optimizations of the item move did not speed up get. Item movement does not dominate get time (0001, 0002, 0003).
- Stacking scan-resistance mechanisms (reference bit plus low insertion) lost on both hit rate and speed (0003). They do not compound.
- Pushing insertion all the way to the LRU end starves new entries of the chance to prove reuse.
- Per-slot frequency counters cost latency on every hit and put for only a marginal gain.
- Isolated latency gains with no mechanistic reason (put in 0001, one variant in 0002) were treated as noise or artifacts.

## Open questions

- Where does median get time go (hashing, bucket lookup, dispatch)? It has been unmeasured for three cycles and is the main untouched suspect.
- Can mid-insertion keep its hit-rate gain at lower put cost, without shifting half the bucket?
- Do the policy gains hold on workloads other than workload-a, or are they specific to its scan-heavy traffic?
- Is the Winner label reliable for marginal variants (0003 v001 looked slightly worse than its parent)? Check the noise floor.
- Is there any untested policy family (ghost/admission filter, adaptive insertion) that offers more than the saturated position axis?

## Per-policy notes

- Baseline LRU: insensitive to bucket width. Partial promotion hurts because the not-fully-promoted hit item becomes the next victim.
- CLOCK reference bit: cheap and helpful alone (removes reordering). It is too coarse as a second chance when stacked on low insertion.
- Frequency counter (2-bit with LRU tie-break): small hit gain, but it adds per-hit and per-put work. It is dominated by cheaper options.
- Low-position insertion (LIP/BIP-style): clearest hit-rate gain alone. A rare hashed MRU escape added branch cost with weak evidence of benefit.
- Mid-bucket insertion with transpose-by-one promotion: the current best policy baseline. Its cost is the median put shift.
