# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle 0001's 24/8 topology. v001 stacked MRU-end promotion with a stricter probing trigger and mid-stash demotion. v002 stacked the frequency-aware LRU-tail victim with stash hits counted. v003 pushed the split to 28/4. v002 was the Winner: it was the only variant that improved all four trace/size cells with no regression, and its largest gain came at the small size, where topology alone had not helped.

## Patterns

- Changes that act inside a bucket (which item gets evicted, where a promoted item lands) are what move the small cache size. Topology changes do not. This confirms cycle 0001's call to stop topology-only work at the small size.
- The frequency filter's gain adds cleanly to the topology gain. MRU promotion adds to it on hm_0 but costs a little on web_0. So the two cycle-0001 levers that looked complementary do not stack equally well.
- With admit-everything-to-stash, the stash is the only way into the protected tier, not just a scan filter. Shrinking it starved promotion on reuse-heavy hm_0 before it hurt scan filtering on web_0. The normal/stash ratio has peaked at about 3/4.
- hm_0 is still the most responsive trace in both directions: it shows the largest gains and the largest regression.
- Cost is still unmeasured. No variant re-measured per-op-time or metadata-size, and the Winner adds an eviction-window scan and bucket-wide halving to the eviction path.
- v001 changed three things at once, so its web_0 loss cannot be pinned to one of them.

## Open questions

- What does v002's 8-slot victim scan and halve-on-cap ageing cost in per-op-time, and would a narrower window or a lower cap keep the hit-rate gain?
- Does MRU promotion still add on top of v002's frequency filter, or does the filter already capture the same small-cache gain on hm_0?
- Which part of v001 causes the web_0 loss: mid-stash demotion or the hit-bit probing guard?
- Would a longer stash wait, or promotion on first re-reference, help hm_0's long-gap reuse without growing the stash?

## Continuation

Use v002 as the parent and make per-op-time measurement mandatory. Tune the scan window and counter cap if the cost is high. Then try adding MRU promotion alone, without v001's demotion and trigger changes, and isolate those two pieces only if web_0 regresses. Drop further normal/stash ratio sweeps. Any later gains should come from admission or stash-promotion changes, not from the split.
