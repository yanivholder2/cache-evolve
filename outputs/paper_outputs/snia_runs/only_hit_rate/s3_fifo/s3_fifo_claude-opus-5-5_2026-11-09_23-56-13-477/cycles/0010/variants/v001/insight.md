# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384): The window drops from 512 to 128 small insertions, so some of web_0's quick reuses now land outside it. Those keys reach freq 2 and graduate directly, instead of being capped at freq 1 and paying a ghost round trip. The recovery is only marginal, so most of web_0's near-insertion reuses must arrive within 128 insertions and are still capped.
- hit-rate (web_0 @ 131072): The same mechanism (window shrinks from 4096 to 1024 insertions) gives a similarly slight uptick. web_0 at the large size stays essentially on its plateau.

## Hurt

- hit-rate (hm_0 @ 16384): Some of hm_0's burst-only keys repeat between 128 and 512 small insertions after admission. Under the narrower window those repeats count as real reuse again, so the keys graduate into a full main and force `evict_main` churn that 0009-v003 had avoided. hm_0's bursts are therefore not confined to "a handful of requests", as the rationale assumed.
- hit-rate (hm_0 @ 131072): The same leakage of burst keys into main, at smaller scale. Fewer full-main graduations are decided at this size, so less churn comes back.

## Takeaway

Going from `CRP_DIVISOR` 8 to 32 moved web_0 and hm_0 in opposite directions by similar small amounts, and the overall result was slightly negative. This means hm_0's correlated bursts and web_0's real quick reuses overlap in the same post-insertion distance band. Narrowing the window only slides along the trade-off and does not break it. Tuning the divisor further is unlikely to pay off. Telling the two apart would need a signal other than insertion distance, such as the hit count inside the window or whether the key returns after leaving small. Otherwise the 0009-v003 divisor of 8 should be kept, since it has the better balance.
