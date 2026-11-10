# Cycle 0010 Insight

## Cohort outcome

All three variants built on 0009-v003, the correlated-reference (CRP) cap. v001 narrowed the CRP window. v002 exempted weak ghost returnees from the cap. v003 was the one allowed web_0 large-size probe: a gated main-victim ghost that readmits returnees with freq 1. v002 won, but it is close to a no-op. It moved web_0 at the small size up by a trace and slipped by a hair everywhere else. In practice, 0009-v003 is still the best policy.

## Patterns

- The hm_0 / web_0 trade-off held for a fifth cycle. v001 and v003 each moved the two traces in opposite directions. v002 was the only variant that didn't, and that is because it barely changed anything.
- The 0009-v003 regression on web_0 at the small size comes from first-time keys. hm_0's burst repeats and web_0's real quick reuses fall in the same distance band after insertion, so the window size cannot separate them (v001). Returnees are not the cause either (v002). Neither insertion distance nor ghost history can tell these keys apart, so a fix needs a new signal that exists on a key's first admission.
- web_0 at the large size does respond after all, but only to how main victims are readmitted, and v003 moved it the wrong way. The ungated main ghost in 0009 did nothing. The gated one with a freq-1 skip past small actively hurt web_0, yet it helped hm_0 at both sizes and web_0 at the small size. So the readmission path is a real lever. On web_0 its gate is stale: `main_hit` stays set across every second-chance pass.
- v003 is the first variant in several cycles to gain on three of the four cells. Its only loss is the cell it targeted.
- Per-op-time and metadata-size are still not measured, for the tenth cycle running.

## Open questions

- Would a recency-scoped `main_hit` remove v003's web_0 pollution at the large size and keep its other gains? Under this change the flag would clear on each second-chance decrement. A gentler readmission, either freq 0 or entry via small, is the alternative.
- Is there any signal available at first admission that separates hm_0's burst-only keys from web_0's real quick reusers? Candidates are the hit count inside the window and whether main is full at the time. Or is that trade-off inherent to the cap?

## Continuation

Keep 0009-v003 as the base, not v002. Spend one hypothesis on fixing v003's readmission: make `main_hit` recency-scoped, readmit with freq 0, or both. This is the most promising thread, because it already gains on three cells. Drop CRP divisor tuning and returnee exemptions from the cap. Any further CRP work should use a signal available on first admission, such as the hit count inside the window, not insertion distance.
