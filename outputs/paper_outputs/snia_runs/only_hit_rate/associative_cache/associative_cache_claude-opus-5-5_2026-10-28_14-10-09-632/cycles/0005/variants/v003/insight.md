# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 16384): The new `PROBATION_FLOOR` doubles the probation ring in 128-slot buckets. A new item now survives twice as many misses before `small_hand` comes back to its slot, so more of web_0's late second touches land while the block is still in probation. Those blocks get promoted instead of being discarded as one-touch. The gain is real but modest: it clears the refutation threshold, yet ring depth explains only part of the small-size gap that v002's insight flagged.
- hit-rate (hm_0 @ 16384): hm_0 improved too, even though the main region in small buckets shrank to half the bucket. This is the opposite of the LRU lineage, where deeper probation cost hm_0. In this design, main counters are decremented only on promotions, so a 64-slot frequency-ranked main region still holds hm_0's hot set. The longer probation window also lets more of hm_0's reused blocks qualify for promotion in the first place.

## Hurt

None. Both 131072 cells are identical to v002's, as expected: `probation_len(1024)` is still 256, because `max_size / 4` already exceeds the floor. So the large-bucket configuration is untouched.

## Takeaway

In the GCLOCK+probation design, probation depth and main-region capacity are not a zero-sum trade-off at 128-slot buckets. Doubling probation at the cost of half of main helped both traces, because the frequency-ranked main with promotion-only decrements needs fewer slots than LRU's protected segment. The hypothesis's refutation case did not happen: hm_0 did not trade away its hit-rate, so ghost or dueling adaptation is not yet required. The web_0 gain is small, though, so the next lever at small sizes is probably not more depth alone. Two candidates are pushing the floor further (such as above half the bucket) and keeping a non-zero counter on promotion. Per-op-time is still unmeasured, even though the hypothesis asked for it. Main sweeps in small buckets now cover a 64-slot region instead of a 96-slot one.
