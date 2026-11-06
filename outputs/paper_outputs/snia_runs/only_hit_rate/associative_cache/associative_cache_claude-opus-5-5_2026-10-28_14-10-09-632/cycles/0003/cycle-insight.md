# Cycle 0003 Insight

## Cohort outcome

Two variants extended cycle 0002's bimodal-insertion winner. v001 replaced the fixed MRU-sample rate with a per-bucket PSEL feedback counter. v003 added the absolute probation floor. The third variant, v002, added ghost-ring warm admission to the GCLOCK branch. The Winner is v003, but it only moved small-bucket cells, and it moved them as a trade: web_0 gained and hm_0 lost. No variant produced a clean gain on both traces.

## Patterns

- All three variants added a short per-bucket memory (a sample ring, a ghost ring, or a fixed probation depth), and in each case that memory was sized below the reuse distance it was meant to catch. In both ring-based variants the memory forgot items before they could produce a signal. The failures were mostly about sizing, not about the ideas.
- The floor and MRU sampling do not stack. Both split the same fixed bucket space: making probation deeper shrinks the protected region where long-reuse hm_0 blocks survive. This confirms the trace split from cycle 0002 at the level of mechanism. hm_0 wants a large protected region and web_0 wants a deep probation region.
- Every change was inactive or harmful in the large buckets. The floor never engages there. The short sample ring biased PSEL toward aggressive MRU sampling, which made web_0 pollution worse. Warm admission without enough memory polluted GCLOCK. Nothing this cycle improved the large size.
- Feedback signals were miscalibrated as well as too short. PSEL rewarded any re-hit, including hits that probation insertion would have caught anyway. Warm credit added only one sweep of extra protection.
- For the third cycle in a row, per-op-time and metadata-size went unmeasured. This cycle added more unmeasured cost: linear scans on the miss path for both rings.

## Open questions

- Would true set dueling (leader buckets fixed to the v001-style depth and the v002-style depth, followers taking the winner) settle the probation/protected split per workload, where per-bucket self-feedback failed?
- Does PSEL feedback work once the tracker covers a full bucket lifetime (for example a per-item sampled bit) and credits only hits after probation?
- Is ghost admission on GCLOCK worth pursuing with a ring that covers a full rotation and a larger credit, or should that direction be dropped?

## Continuation

Pursue adaptive selection of the probation/protected split, preferably leader/follower set dueling between cycle 0002 v001's and v002's insertion depths, with no fixed combined depth. Any feedback state must be sized to the bucket's lifetime. Keep the GCLOCK branch, but try ghost admission only once more, with full-rotation coverage; drop it if that still shows no effect. Stop stacking the floor onto BIP, and require per-op-time measurement.
