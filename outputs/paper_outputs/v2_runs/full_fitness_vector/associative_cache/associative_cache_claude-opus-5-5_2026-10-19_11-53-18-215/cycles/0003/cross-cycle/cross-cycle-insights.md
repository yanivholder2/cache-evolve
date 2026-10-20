## Recurring wins

- **One shared policy per merged bucket.** This has been the base since cycle 0001 (v003) and has paid off in every cycle since. Each key hashes once to one physical bucket, and the bucket uses one shared policy. This passes P2 slot_legality and gives the best hit rates seen so far.
- **Cheaper mechanics give speedups that carry across policies.** One hash per op, contiguous bucket storage and in-place policy updates helped in cycle 0001. Narrower items and shorter moves (cycles 0002 and 0003) also helped. These changes made get and put faster without changing hit rate, so they can be stacked freely.
- **Latency comes from data movement.** Each cycle found the same cause: memmoves of whole items across the bucket. Three things kept helping:
  - smaller `Item`s (the 24-byte logical-tick form)
  - scanning from the MRU end
  - keeping eviction rotates inside probation
- **Smaller metadata costs nothing under position-ordered policies.** Under LRU and SLRU the `timestamp` and `lfu_counter` fields are never read. Shrinking them made metadata smaller and put faster, with hit rate flat.
- **No wall-clock reads on the hot path.** Replacing `Instant::now()` with logical ticks made put and the tails faster.
- **Scan resistance adds hit rate on this LIRS-style trace.** SLRU beat its parent on hit rate. It needs no extra per-item metadata because the segment boundary is stored in the existing length field.

## Recurring failures

- **Placing a key outside its hashed bucket.** Two-choice, cuckoo and cross-bucket victim designs failed P2 slot_legality on every attempt. Do not propose them again.
- **Splitting LRU into home and partner halves inside a bucket.** It lowered hit rate and added tail cost on the partner path. Removing the split recovered hit rate in cycle 0002.
- **Rewarding a ghost hit with protected status.** In cycle 0003, sending returning ghost keys straight to protected broke scan resistance and lost hit rate. Protected is usually full, so each such insert pushes out an item that earned its place with a real second hit. A ghost hit is weaker evidence than a resident second hit.
- **Trusting small latency changes without a clear cause.** Some get-median and per-op shifts appeared with no explanation in the code and are probably noise. Re-measure them before giving any design credit.

## Open questions

- **Stacking the cycle 0003 wins.** Combining the 24-byte `Item` with MRU-first `find` and the probation-only rotate has not been tested yet. They should add up.
- **Removing the leftover protected-wide moves.** Two moves still cross all of protected:
  - the re-find after insert, which could be avoided by returning the index from put
  - the rotate on a probation-to-protected promotion
  
  Ring offsets or swap-based promotion might remove both.
- **Plain LRU vs SLRU on the same base.** This has been deferred for two cycles. So has tuning the 3/4 protected fraction.
- **Probation-only use of history.** Can ghost history help hit rate if it only affects probation or an ARC-style adaptive split, and never protected?
- **A tag or fingerprint array scanned before full items.** Would it lower the get median further?
- **Removing `timestamp` and `lfu_counter` entirely on the SLRU path.** Would this shrink metadata further without hurting other policy kinds?

## Per-policy notes

- **Split-half LRU (home/partner):** Superseded. It lowered hit rate and added tail cost on the partner path.
- **Plain LRU over a merged bucket:** It recovered hit rate above baseline in cycle 0002. It has never been compared directly with SLRU on a matching base.
- **SLRU (probation/protected, 3/4 protected):** The current best policy and the current base. It gains hit rate from scan resistance. Its cost is in moves that touch protected, so keep evictions inside probation. The protected fraction has not been tuned.
- **SLRU + ghost-to-protected:** Lowered hit rate, so drop it. Ghost signals should only influence probation or an adaptive split.
- **Timestamp / LFU-based policies:** None have been tried inside the single-bucket rule. If one is tried, it would need the metadata fields that LRU and SLRU leave unused, so check whether it can still use the compact `Item`.
