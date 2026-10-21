# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The new `should_admit` jitter lets a warm candidate (estimate ≥ 6) that loses or ties replace the victim with 1/128 probability. But the victim is always the min-estimate entry among the 8 oldest *probation* entries (`select_victim` is unchanged), not the stale-high resident the hypothesis aimed at. Each jitter win therefore evicts a probation key whose lifetime count was at least as high as the newcomer's, and does nothing to the protected-segment keys that actually hold old popularity. The result is a small net loss.

## Takeaway

Random admission jitter cannot break lifetime-count lock-in in this SLRU design. The admission contest only ever faces probation entries. Keys with stale-high counts sit in the 80% protected segment, and they leave only through LRU demotion, never through the contest. So the jitter only adds churn among already-cold probation entries, and it trades away a key with an equal or higher count for one with a lower count. To attack stale residents, the change has to act on how protected entries get demoted or reach the victim sample, not on the admission coin-flip. At a 1/128 rate the effect is small either way.
