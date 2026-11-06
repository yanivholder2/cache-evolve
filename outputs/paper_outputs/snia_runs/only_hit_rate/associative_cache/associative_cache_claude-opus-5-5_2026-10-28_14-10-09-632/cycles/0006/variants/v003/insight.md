# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 131072): The target cell broke out of its plateau from cycle 0004. With `GCLOCK_MAX = 7`, main counters only fall during promotion sweeps. So a block hit many times since the last sweep now survives several more sweeps than one hit three times. In the 768-slot main region of large buckets, the sweep can finally tell hm_0's hot blocks from its warm ones. Before, both looked the same at the old ceiling of 3. The gain is still below what cycle 0005 v001's near-miss retention reached. Frequency resolution recovers some of that reuse, but not all of it.
- hit-rate (hm_0 @ 16384): The same effect, smaller. The 64-slot main region in 128-slot buckets also gets finer ranking, but it has fewer residents, so there are fewer warm-vs-hot ties to break.
- hit-rate (web_0 @ 16384, web_0 @ 131072): These cells edged up slightly instead of dropping. This is the opposite of the refutation case: long-lived counters did not trap stale web_0 blocks in main. Promotion resets counters to 0, so an item must earn its high counter after promotion, and web_0's stale blocks rarely get re-hit enough to reach the higher levels.

## Hurt

None on hit-rate. Per-op-time is still unmeasured. The worst-case promotion sweep is now up to 7 rotations of main instead of 3, and this variant gives no evidence of what that costs.

## Takeaway

A ceiling of 3 was a real but small limit on main's retention. Widening it to 7 helped hm_0 most in large buckets and did not hurt web_0. That shows the promotion-reset-to-0 rule protects against stale counters, so a higher ceiling is safe to use here. The small size of the gain suggests main-side frequency resolution is close to used up as a lever. Pushing the ceiling much higher would mostly add sweep cost, not hits. The remaining hm_0 gap to v001 more likely needs recency signals that a frequency counter cannot capture.
