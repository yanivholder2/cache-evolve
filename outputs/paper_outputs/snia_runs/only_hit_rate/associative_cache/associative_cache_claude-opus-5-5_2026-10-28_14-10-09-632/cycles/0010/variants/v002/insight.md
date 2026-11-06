# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): The one-lap retention rule clears the refutation bar, but only just. In 128-slot buckets the correlated window is only 4 slots (`64 / CRP_DIV`), so few probation slots end up recent-only. The few that do get a second chance at a non-correlated hit, and that is enough for a small positive effect without disturbing anything else.
- hit-rate (web_0 @ 16384, web_0 @ 131072): Both cells stay at or above v001, so the worry in the refutation clause (that the extra lap costs web_0 cold slots) did not happen. web_0's burst-only blocks get no hit during their extra lap, so they are still evicted on the hand's next visit. The small gain at 131072 suggests that some of web_0's burst blocks do return within one more lap.

## Hurt

- hit-rate (hm_0 @ 131072): This cell falls slightly below v001, which refutes half of the claim. In 1024-slot buckets the correlated window is 16 slots wide, so more probation slots get the recent bit, and the walk skips them more often. Each skip advances `small_hand` past an extra slot on the same miss. Because probation age is position-based, every never-hit neighbour after a skipped slot reaches the hand after fewer misses. So the extra lap for recent-only slots is paid for with shorter probation for everything else, and on hm_0 at this size those neighbours were worth more.

## Takeaway

Retention in a position-based CLOCK probation ring is zero-sum: each lap granted to a recent-only slot takes residency from the slots behind it. The net effect therefore depends on how wide the correlated window is compared with the ring. It is a slight gain when the window is 4 slots (128-slot buckets) and a slight loss when it is 16 slots (1024-slot buckets). The size-limited fallback that the hypothesis itself suggests (apply the rule only when `small_len` is small) is the obvious follow-up, though the hm_0 @ 16384 gain is too small to make this a strong lever.
