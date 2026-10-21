## Recurring wins

- Low protection for new keys on insert is the main hit-rate lever on workload-a. It has won in all three cycles (0001–0003) and helped both the shift-based LRU line and the in-place CLOCK line. New keys must earn protection with a second hit.
- Making the newcomer filter stronger has kept paying off. Insertion moved from the midpoint to 1/4 and then to 1/8, and the CLOCK hand now rests on the newcomer. Even a grace window of about one to two misses has not hurt hit rate on this trace.
- Exact LRU ordering inside a bucket is not needed. In-place CLOCK/GCLOCK beat the baseline in every cycle, and in Cycle 0003 it nearly matched deep-insertion LRU once its newcomer filter was equally strong.
- Shrinking the item record had no effect on hit rate (Cycle 0001), so the per-item wall-clock timestamp can safely be removed.

## Recurring failures

- Extra protection beyond the newcomer filter keeps adding little or nothing. Sending ghost hits straight to MRU lost hit rate (Cycle 0002). Picking victims by frequency at the eviction end was within noise (Cycle 0003). One recurrence does not predict a near-term hit on this trace.
- Frequency counters inside the bucket (GCLOCK) helped plain CLOCK a little (Cycle 0002), but did not close the gap. The gap closed only once the hand placement filtered newcomers.
- Per-op-time and metadata-size have not been recorded in any cycle. As a result, every latency or size hypothesis is still untested, and the main ranking tradeoff cannot be settled.
- Each depth step gains less than the one before it: 1/4 to 1/8 gained less than midpoint to 1/4. Hit-rate tuning along the depth axis is close to its limit.

## Open questions

- Does pure LIP (index 0) improve on 1/8, or has the depth curve already peaked?
- Once per-op-time is recorded, does in-place GCLOCK with the hand resting on the newcomer beat deep-insertion LRU overall? The LRU line pays more element shifts on every miss as depth grows.
- Is the aggressive newcomer filter over-fitted to workload-a? Every result so far comes from one trace, with no checks at other cache sizes or reuse distances.
- Could frequency help on the insertion or promotion side, for example by setting insert depth from a key's count, even though it failed on the eviction side?
- Does a shared global atomic clock (from the smaller-item variant) hurt multi-threaded throughput?

## Per-policy notes

- **Shift-based LRU with an insert depth (midpoint, 1/4, 1/8):** won hit-rate in every cycle, with each gain smaller than the last. Its miss-path cost (shifting part of the bucket on every miss) grows with depth and has never been measured.
- **CLOCK / GCLOCK (in-place):** beat the baseline in all three cycles. The single bit lost useful signal, and the saturating counter helped. The biggest gain came from leaving the hand on the new key. This line has the strongest latency case, but that case is unverified.
- **Ghost list with MRU admission:** hurt hit rate. It has been dropped and should come back only in a much more conservative form, such as a partial promotion or a small, recent ghost ring.
- **Frequency-aware eviction window on deep-insertion LRU:** gain was within noise. Drop it.
- **Logical u32 stamps / smaller Item:** hit rate unchanged. Its size benefit still needs metadata-size to be measured. It could be combined with the CLOCK line.
