## Recurring wins

- Cold insertion has won in every cycle in some form. New items enter below items that have already been reused (LIP in 0001; LIP→BIP in 0002–0003; a probation FIFO in front of GCLOCK in 0004). How a new item ranks against a reused item is the main thing that sets hit-rate on these MSR traces.
- Coarse recency costs no hit-rate. CLOCK (0001) and GCLOCK (0002) did as well as or better than exact LRU, and the frequency-aware CLOCK line's advantage grows with bucket size.
- Admission filtering that only promotes on a hit during probation (0004) is the only mechanism so far that helped both hm_0 and web_0 without a trade-off. It is also the first change since 0001 to clearly move the large-bucket cells.

## Recurring failures

- Fixed splits of one bucket into probation and protected regions keep trading one trace for the other. hm_0 needs a large protected region for long reuse. web_0 needs deep probation for late second touches. Fixed floors, fixed BIP periods, and floor+BIP stacking (0002–0003) all trade along this axis.
- Fixed-fraction probation (len/4-style) is too short in the 128-slot buckets for web_0's late reuse. This held in cycles 0001–0004 across both lineages.
- Feedback and memory mechanisms keep failing on detail, not on the idea. Rings sized below the reuse distance lose the signal (0003). Symmetric, mid-start PSEL counters vote on the wrong question and drift toward LIP (0004). Rewards count hits that probation would have caught anyway (0003).
- Promotion tweaks that don't change where new items land have not helped: bounded promotion (0001) and graded promotion through a single shared boundary slot (0004).
- Changes tuned for small buckets usually do nothing or do harm at the large size (0002–0003).
- per-op-time and metadata-size have never been measured in four cycles, though hypotheses keep asking for them. Every cost claim is still untested: mid-vector inserts, GCLOCK sweeps, ring scans, and miss-path linear scans.

## Open questions

- Can the probation size adapt (a size that scales with the bucket, set dueling, or an S3-FIFO-style ghost on the probation size) so that web_0 recovers at the small size without weakening large-bucket filtering?
- Should promoted items enter main with a non-zero counter, so they are not the next sweep victim?
- Is true leader/follower set dueling a better fit than per-bucket self-feedback for choosing the probation/protected split?
- Can the LRU/BIP line be saved by tag-based PSEL with an asymmetric update, a start in the higher-sampling band, and promotion-aware rewards? Or should it be retired?
- What do the GCLOCK, probation-ring, and ghost-ring mechanisms actually cost in per-op-time?

## Per-policy notes

- LRU + LIP/BIP: strong first win (0001), then only small trace-split gains (0002–0003), then stuck (0004). Every extension has traded hm_0 against web_0, and the line is now secondary.
- CLOCK / GCLOCK: as good as LRU from the start, and it has improved every cycle it was extended, apart from ghost-ring warm admission, which was undersized and polluted (0003). It does no element shifting and scales best to large buckets.
- GCLOCK + probation FIFO (0004 v002): current main lineage. It filters scans on web_0 and protects hm_0's reused blocks. Its known gap is web_0 at the small size, where probation is too short.
- Adaptive insertion (PSEL variants): failed in both 0003 and 0004. The tracker covered too little in 0003 and the update rule was miscalibrated in 0004. Untested: a version with lifetime-length state and asymmetric updates.
