# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 16384): The extra small-queue lap lets single-reuse keys survive one more pass while main is full and graduate on a second reuse. This is the medium-reuse keys the hypothesis targeted, which the parent would have ghosted. The gain is real but marginal.
- hit-rate (web_0 @ 16384): web_0 at the small size also ticked up slightly instead of being hurt by small-queue churn. Delaying ghosting does not cost new keys enough small capacity to matter at this size, and main admission stays strict.

## Hurt

- hit-rate (web_0 @ 131072): This cell dipped slightly. One likely cause: at the large size, a relapped key graduates on `freq >= 1` after its lap, so main admits keys whose two reuses are spread over a much longer window. That is weaker evidence than the parent's two reuses within a single small residency. The relap also makes `evict_small` ghost the next tail early, which cuts newcomers' small residency short. Because the dip is small, this explanation is a guess, not something the eval shows.

## Takeaway

Moving the "prove a second reuse" window from the ghost into small (one relap per key, only while main is full) is roughly hit-rate-neutral. hm_0 and web_0 gain slightly at 16384, web_0 loses slightly at 131072, and hm_0 at 131072 is unchanged. So the hypothesis's direction holds at the small size, but the effect is much smaller than the earlier "pollution vs. filtering" moves. Its predicted failure mode (web_0 losing to small-queue churn) did not happen at the small size. The idea is probably limited by how few freq-1 tails meet a full main, and making the lap longer or repeatable is unlikely to change much. The large-size dip suggests that a relapped key's second reuse should come with a tighter recency requirement, not just `freq >= 1` at any point in a full extra lap.
