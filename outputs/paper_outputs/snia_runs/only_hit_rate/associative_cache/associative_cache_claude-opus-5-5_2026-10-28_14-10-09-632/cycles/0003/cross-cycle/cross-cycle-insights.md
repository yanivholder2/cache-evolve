# Cross-Cycle Insights (cycles 0001–0003)

## Recurring wins

- Where new items are inserted matters more than anything else. Every Winner so far (0001 v001, 0002 v002, 0003 v003) is in the cold-insertion lineage, which places new items below reused ones. Changes that leave new items at MRU have lost.
- The GCLOCK lineage improves hit-rate reliably. Exact LRU order hasn't been needed: CLOCK matched LRU in 0001, and 2-bit GCLOCK improved on CLOCK in every cell in 0002. The gains are largest in the large buckets, where the insertion lineage has the least room left.
- Small-bucket changes can recover hm_0 reuse that lies beyond the probation window. Deeper probation and occasional MRU insert both did this, though only at the small size.

## Recurring failures

- Fixed rules that apply to every workload split by trace. hm_0 wants a large protected region and web_0 wants deep probation. Both rules split the same fixed bucket space, so a fixed BIP period or a fixed floor helps one trace and hurts the other. The floor and BIP sampling don't stack (0003).
- Per-bucket memory structures sized below the reuse distance they target have failed: sample rings, ghost rings, PSEL trackers, and short probation windows. They forget items before they can produce a signal. Feedback that credits hits probation would have caught anyway is miscalibrated too.
- The large buckets have not improved since 0001 for the insertion lineage. Floors never engage at the large size, MRU sampling pollutes web_0 there, and warm admission without enough memory pollutes GCLOCK.
- Promotion tweaks (bounded or graded promotion) while new items still enter at MRU: these made hm_0 worse.
- In three cycles, nobody has measured per-op-time or metadata-size, and every cycle has added more unmeasured cost: mid-vector inserts, GCLOCK sweeps, full-bucket scans, linear scans of rings on the miss path.

## Open questions

- Can true leader/follower set dueling between the shallow and deep probation/protected splits choose the right split per workload, where per-bucket self-feedback failed?
- Does feedback work once it is sized to the bucket's lifetime (for example a per-item sampled bit) and credits only hits after probation?
- Can GCLOCK take a cold-insertion analogue, such as starting new items below referenced ones? Does ghost admission with full-rotation coverage do anything? If not, drop ghost admission.
- What is the real latency cost of each lineage? In particular, does CLOCK/GCLOCK beat mid-vector LRU insertion once worst-case sweep cost is counted?

## Per-policy notes

- **ClassicLRU + cold insertion (LIP-style):** This is the base of every Winner. A fixed-fraction probation depth is too short in 128-slot buckets (it loses hm_0 reuse) and works well in 1024-slot buckets (it drains scans).
- **BIP / occasional MRU insert:** Helps hm_0 at the small size and hurts web_0 at the large size. Tuning the period alone is exhausted. A PSEL rate driven by a short ring pushed it toward over-sampling.
- **Absolute probation floor:** Affects only small buckets and trades web_0 gains for hm_0 losses. Don't stack it on BIP.
- **Bounded or graded promotion:** Harmful while new items enter at MRU. Revisit it only on top of cold insertion.
- **CLOCK → GCLOCK:** A credible second lineage with no element shifting and the strongest scaling to large buckets. Ghost-ring warm admission had no effect at its current sizing and credit. It gets one more try with full-rotation coverage.
