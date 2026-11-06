# Variant v001 Insight

> Note: this session's tools could only read files at known paths, so the overlay
> source and eval/ contents could not be listed or inspected. The analysis
> below is based on the hypothesis (H01), the parent's insight, and the two
> fitness vectors. Mechanism claims are inferred from the hypothesis design, not
> checked against the diff.

## Helped

(none — no hit-rate cell improved over parent v002)

## Hurt

- hit-rate (hm_0, size 131072): The largest regression. The exemption hides the min-counter candidate from eviction exactly where v002 gained the most. At this size ghost hits are frequent, so the min-counter slot in the 8-slot tail window is often a flagged counter-1 arrival. The scan then evicts the next-best candidate instead, which is a resident with a higher counter that has already proven reuse. Each saved credited key costs a proven key.
- hit-rate (web_0, size 131072): Fell by a similar amount. This undid v002's unexpected gain here. web_0's ghost-returners mostly do not come back a second time, so protecting them evicts useful residents with nothing gained in return.
- hit-rate (hm_0, size 16384): Fell slightly. This is the opposite of the hypothesis's main prediction. One exemption delays a credited key's eviction only until the next scan of the same window. In a small, contended target that is too short to reach the key's next reuse. The displaced runner-up is still a real loss.
- hit-rate (web_0, size 16384): Small decline, near noise. Few ghost hits fire here, so the exemption rarely triggers. It still misdirects a few evictions in the `demote_from_normal_bucket` cascade.
- Possible confound (unverified): if the flag took the counter's top bit and lowered the counter cap, cap-triggered halving happens sooner. That would weaken frequency discrimination across all residents, not only ghost-admitted ones. This fits the regression being broad across all four cells. Check the overlay's counter cap before reusing this design.

## Takeaway

A one-shot exemption from the tail scan does not move eviction pressure away from ghost-credited keys. It moves it onto the next-lowest resident, which on these traces is usually worth more than an unproven returner. The losses are largest at the large size, where v002's plain Front/counter-1 landing was already working. v002's fast eviction of un-re-hit ghost arrivals was a feature, not the bottleneck. Future work on the 16384 gap should add target capacity or change admission, not protect credited keys inside the victim scan.
