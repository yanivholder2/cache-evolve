## Recurring wins

- Structural changes to the admission/eviction contest keep paying off: the 1% window (c1), adaptive window (c2), SLRU main region (c3), and sampled probation victim (c5) were all clear wins. Each one changed who competes, or which resident is the victim.
- Sharper frequency-based selection is the dependable source of gains, mostly on web_0. Examples: SLRU, the coldest-of-N victim, and less sketch collision noise.
- Strict `>` admission, where the victim wins ties, is confirmed better than recency or `>=` tie-breaks (c4). Ties only became a problem when a counter cap or halving made them common.
- Building in steps on the current winner, one axis per variant, has kept attribution clear. The lineage now runs: window → adaptive window → SLRU → adaptive protected split (marginal) → sampled victim.

## Recurring failures

- Changes to feedback-loop or controller timing have failed every cycle from c3 to c5. This covers climber sample periods (capacity-tied, constant, and turnover-tied), alternating dual climbers, and the ARC-style ghost controller. The causes repeat: noisy deltas, lazy rebalancing that hides the effect of a retarget, integer step truncation, warmup bias, and unbalanced ghost inputs that ratchet the window down to its minimum.
- Periods tied to `capacity` (aging reset, climber sample) misbehave at one end of the size range or the other.
- Global periodic sketch halving helped on its own (c2) but hurt once combined with the adaptive window (c3). It wears down the main region's frequency lead and creates ties. Fixing how the period was counted did not remove the interaction.
- Counter caps combined with strict `>` cause tie-driven rejections of hot newcomers (c1).
- Hypotheses meant to rescue recency-driven entries on hm_0 have repeatedly had their stated mechanism refuted (c3–c5).
- Metadata-size and per-op-time have gone unmeasured for five cycles in a row, despite every continuation asking for them. Every "win" so far is a hit-rate-only win.

## Open questions

- Does single-count sketch accounting (the correct base) add to the sampled-victim win? The two gains come through different routes.
- How far can the victim sample size grow before the per-op estimate cost outweighs web_0 gains?
- Can hm_0 be helped on purpose through an explicit re-entry/ghost bonus for recently evicted keys, or by changing where admissions land in probation or how long they stay?
- Is the 80/20 protected split already near optimal? The adaptive split's win was marginal.
- Is controller work worth resuming at all? If so, only with a balanced, batched, counterfactual signal, small steps, and suppression until the cache is full.
- Could aging that targets only stale demoted entries fix inflated counts without broad erosion?
- What do the window, SLRU, climber, and sampling actually cost in metadata and latency?

## Per-policy notes

- TinyLFU baseline: its weakness is the admission contest. Once the cache is full it rejects every first-time key.
- Window (fixed / adaptive): the fixed 1% window is too small at small sizes. Adaptive helps the small size on both traces and does nothing at large sizes. It does not fix web_0 at the large size.
- SLRU main region: drives web_0 gains at both sizes, and was the first change to move web_0 at the large size. It costs hm_0 slightly, because the small probation segment cuts short recency-driven stays.
- Sketch aging/halving: the regression at c1 came from the cap, not the aging. Cap-free aging helps standalone but hurts when combined with the adaptive window. Dropped on this lineage.
- Sketch accounting: a miss followed by an insert used to count twice, which inflated one-time keys and defeated frequency floors. hm_0 small had been relying on that accidental bias. Single-count is correct but costs hm_0 small.
- Victim selection: coldest of the oldest few probation entries is a clean win on web_0. Further victim-choice tweaks at the LRU end are not expected to help hm_0.
- Traces: web_0 is scan-heavy and responds to frequency filtering. hm_0 is recency-driven, gains little at the large size (LRU already captures most of it), and regresses at the small size under almost every change. It is the persistent hard point.
