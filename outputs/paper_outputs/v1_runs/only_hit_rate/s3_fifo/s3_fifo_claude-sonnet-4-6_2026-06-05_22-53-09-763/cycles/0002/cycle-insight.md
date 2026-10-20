# Cycle 0002 Insight

## Cohort outcome

All three variants probed two axes independently and in combination: further small-queue shrinkage and ghost-queue precision via a halved cap. The winner (v003) stacked cycle 0001's enlarged main queue with a tighter ghost window, confirming that both dimensions contribute independently to hit rate.

## Patterns

- **Ghost precision is a first-class lever.** Both variants that halved the ghost cap (v002 on the baseline, v003 on the cycle-0001 configuration) improved hit rate — the ghost truncation worked regardless of which small-queue ratio accompanied it. The benefit is not an artifact of the enlarged main queue; it is a property of the workload's re-use distance fitting inside a smaller window.
- **The two levers are complementary, not redundant.** v003's combined change outperformed v001 (small-only) and, by extension, the cycle-0001 winner. A larger main queue and a sharper ghost signal amplify each other: the enlarged main slows eviction churn, making the freshest evictions colder on average and therefore making ghost precision matter even more.
- **Small-queue shrinkage is hitting diminishing returns.** The 10% → 8.3% step in v001 produced a gain visibly smaller than the large 25% → 10% jump from cycle 0001. The marginal value of each slot transferred from small to main is declining, signalling a plateau in this direction.

## Open questions

- Does the ghost benefit continue below main_size/2 (e.g., main_size/4), or does the window become too narrow and miss genuine re-uses?
- With v003 as the new baseline, does further small-queue reduction (e.g., capacity/12 or capacity/14) still help, or have we exhausted small-queue tuning?
- The promotion threshold (`freq > 1`) has remained constant throughout; with a sharper ghost signal now filtering main-queue entry, is there headroom to explore `freq > 0` without flooding main?

## Continuation

Next cycle should treat v003 as the new baseline and probe ghost sizing more aggressively — specifically a ghost cap of main_size/4 — to find where ghost truncation tips from precision gain to signal loss. A secondary axis worth opening is whether the combination of the sharper ghost and the now-stable small-queue size creates room to revisit the promotion threshold, which has been untouchable until now. Further small-queue-only shrinkage is unlikely to be the highest-yield direction given the diminishing returns seen in v001.
