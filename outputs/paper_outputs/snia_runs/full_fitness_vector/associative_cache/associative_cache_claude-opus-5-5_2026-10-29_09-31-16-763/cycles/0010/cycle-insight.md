# Cycle 0010 Insight

## Cohort outcome

Two variants built on the cycle 0009/v003 single-hash base. v001 stacked top-eighth lazy promotion onto it. v002 removed the `Box<dyn EvictionPolicy>` call from the ClassicLRU promote path. v003 went back to the older 0007/v003 base and shrank `Item` from 24 to 16 bytes by moving policy state into the existing `tags` buffer. All three are Winners. v002 is the clean win: put got much faster, get got slightly faster, and hit rate was bit-identical. v003 moved metadata-size for the first time since cycle 4, but both ops got slower at p50 for reasons nobody can explain yet. v001 cut put time but did not help get.

## Patterns

- The put-update path had more hidden cost than anyone expected. Both v001 and v002 got their largest gain on put, which neither hypothesis targeted. Both skip the same work: the virtual promote call plus the separate `tags` fix-up that `put_lip` reaches through `get_from_position`. Removing dispatch helped put far more than it helped get.
- On get, in-bucket promotion is now close to the floor. With SipHash gone, devirtualisation gave only a small get gain. Lazy promotion gave the smallest measurable p50 step and made p99 worse, probably because its data-dependent branch runs on every hit. The two cycle-9 savings did not add up on get.
- An exact-semantics fast path beats one that changes semantics. v002's "already at back → return" check keeps the order unchanged and has no hit-rate cost. v001's top-eighth window cost a little hit rate on the large-cache cells and added a branch that hurt the tail.
- Shrinking `Item` works structurally. ClassicLRU order stayed exact, the bucket stride stayed at 80, and no field was added. But the expected latency gain from moving smaller items did not appear. The slowdown is most likely codegen fallout from widening the dynamic trait signature, or drift. This is inferred, not shown.
- Hit-rate and stride invariants held again for every mechanical rewrite. As in cycles 7–9, single-run latency moves on unchanged paths still need same-session confirmation.

## Open questions

- Does v003's regression disappear once the lip path no longer goes through the dynamic call, so that the widened signature is off the hot path? Or does it disappear in a same-session A/B?
- On a devirtualised base, do 16-byte items speed up the put-miss fused rotation and insert memmoves, which are now put's main remaining cost?
- What dominates get now: the full-key compare, or bucket indexing (`%` vs mask)?

## Continuation

Adopt 0009/v003 plus v002's devirtualised `lip_promote` as the new base. Then port v003's 16-byte `Item` and side buffer onto that base. This also tests whether the regression came from the dynamic call, and it should be judged by repeated same-session runs against the base. Drop top-eighth lazy promotion and the planned `n/4` window probe. Drop any further get-side promotion work. Next, profile the full-key compare and bucket indexing for get, and the miss/insert memmoves for put. Keep the policy frozen.
