# Variant v001 Insight

## Helped

- hit-rate: Promotions (stash→target, target→probing) now land at the MRU end instead of index 0. A stash hit no longer makes the item the next ClassicLRU victim in its target bucket, so target/probing finally works as a protected tier. The gain is largest at the 16384 size on both traces, especially hm_0. That is where bucket pressure is highest, so the baseline's Front-landing promotions were evicted before they could be reused.

## Hurt

- hit-rate: web_0 at 131072 got slightly worse. The MRU landing changes the probing trigger: an item that just came out of stash already sits at `promotion_candidate_index`, so its next target hit sends it straight to probing. That causes more probing churn and more demotion cascades, and every demotion now goes to the LRU end of stash. At large capacity, web_0's re-references are spread out enough that demoted-but-still-useful items used to survive in stash, and now they are evicted first.

## Takeaway

Splitting the single `insert_pos` into a promote position (Back) and a demote position (Front) works overall. The gain comes from no longer treating fresh promotions as eviction candidates, so it shows up mostly when the cache is small and under pressure. The side effect is that MRU landing also makes the "hottest in target → probing" trigger fire after just one more hit. Future variants should consider pairing MRU promotion with a stricter probing-promotion condition, or a less aggressive stash demotion position, so the large-cache web_0 case doesn't give back what it gained.

Note: this variant's fitness vector only reports hit-rate. metadata-size and per-op-time weren't re-measured, so this change says nothing about them. Only insert positions changed, so neither is expected to move.
