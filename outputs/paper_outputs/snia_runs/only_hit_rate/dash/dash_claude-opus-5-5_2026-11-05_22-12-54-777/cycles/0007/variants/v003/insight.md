# Variant v003 Insight

## Helped

- hit-rate (web_0 size 16384, hm_0 size 16384): Only noise-level movement. In the small cache, target buckets are more often full, so the gate occasionally fires. When it does, it keeps a resident with a slightly higher count in place of a single-hit stash candidate. That is too rare to register.

## Hurt

- hit-rate (web_0 size 131072, hm_0 size 131072): Only noise-level movement in the other direction. The few candidates the gate rejects wait in the stash for a second hit, instead of entering target on their first hit. In the large cache, some of these are real reuse that the parent would have kept a little longer.

## Takeaway

The TinyLFU admission gate on stash→target promotion is close to a no-op in this design, for two reasons. First, the hypothesis aimed at the wrong population. A one-hit scan key never gets a stash hit, so it never reaches `promotion_get`'s stash branch, and the gate never sees the web_0 scan traffic it was meant to block. Second, the gate rarely rejects anything. Cap-15 halve-on-cap ageing, Front-landing promotions, and v003's demotion halving keep target-tail counters near 1, so a fresh stash hit (counter ≥ 1) almost always ties or beats the 8-slot tail victim, and ties admit. Promotion admission is not where web_0 loses hits. The hypothesis's suggested loosening to `cand + 1 >= victim_cnt` would only weaken a gate that barely fires, so it is not worth testing. The 8-slot scan per stash hit adds cost and gains nothing, so this change should be dropped rather than tuned.
