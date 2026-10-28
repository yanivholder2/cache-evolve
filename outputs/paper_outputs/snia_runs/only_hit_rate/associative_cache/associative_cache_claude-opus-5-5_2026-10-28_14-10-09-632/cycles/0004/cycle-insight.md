# Cycle 0004 Insight

## Cohort outcome

Two variants built on cycle 0002 v002 (LRU + bimodal insertion). v001 drove the MRU-sample rate with a PSEL counter fed by per-item tags. v003 added graded two-step promotion. The third, v002, put an S3-FIFO-style probation ring in front of the cycle 0002 GCLOCK bucket. v002 is the Winner. It improved hm_0 at both sizes and gave web_0 at the large size its biggest gain, beating that hypothesis's refutation threshold. web_0 at the small size stayed flat. This is the first cycle since 0001 where the large-bucket cells clearly moved.

## Patterns

- Cold insertion carries over to CLOCK, and it works better there than in the LRU lineage. In v002, a new item only gets into the frequency-ranked main region if it is hit while in probation, and the main sweep only runs on promotions. That filtered scans on web_0 and also kept hm_0's reused blocks protected, so it is the first change to help both traces without a trade-off. The two lineages have now traded places: GCLOCK+probation leads, and the LRU/BIP line is stuck.
- Both LRU-lineage variants traded one trace against the other again, and each failure came down to mechanism detail, not the idea itself. v001's PSEL fixed cycle 0003's lost-signal problem: per-item tags last for the item's whole lifetime. But the counter's symmetric ±1 update, starting at mid-range, votes on whether *most* samples hit, not on whether sampling pays off. So it drifted toward LIP and gave back hm_0's gain. In v003, every lower-half hit landed in one shared slot, so the protected half was cut off from the reuse stream.
- Every variant still runs into the same limit: fixed-fraction probation is too short in 128-slot buckets for web_0's late second touches. That held for the LIP lineage in cycles 0001–0003, and it now holds for v002's `len/4` ring too.
- For the fourth cycle in a row, per-op-time went unmeasured, even though all three hypotheses asked for it.

## Open questions

- Would a probation ring that is deeper in small buckets, or that adapts its size (S3-FIFO-style ghost or set dueling on `small_len`), recover web_0 at the small size without weakening the filtering in large buckets?
- Should promoted items enter main with a non-zero counter, so they are not the next sweep victim?
- Can v001's tag-based PSEL be saved with an asymmetric update, a start in the higher-sampling band, and a reward test that also counts promotions, or should the LRU/BIP line be retired?

## Continuation

Make cycle 0004 v002 (GCLOCK + probation FIFO) the main lineage. Tune or adapt `small_len`, aiming at web_0 at the small size, and try giving promotions a non-zero starting counter. At most one LRU-lineage variant should be kept: the asymmetric-PSEL fix. Drop graded promotion through a single boundary slot. Per-op-time must be reported, or the variant is not accepted.
