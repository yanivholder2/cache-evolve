# Cross-Cycle Insights

## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier is the
  dominant winning lever; it has topped its cohort in consecutive cycles.
- The protected-tier curve has not saturated: pushing the split past even
  toward normal-dominant keeps gaining hit-rate, implying the baseline stash
  was over-provisioned by more than one step.
- Winning moves stay budget-conserving and invariant-preserving — gains came
  with no observed regression on other dimensions.

## Recurring failures

- Stacking selective stash-admission gates (frequency gate, back-insertion)
  on top of a capacity reshape is redundant, not additive: once the protected
  tier is enlarged, layered admission filters give steeply diminishing
  returns. Most benefit is captured by *how much* room the protected tier gets.
- Pure single-lever re-runs of the baseline tiering have been exhausted as a
  direction; the productive axis is now bracketing the split optimum.

## Open questions

- Where does the normal-dominant split stop paying off — is there a point where
  too small a stash starves admission and regresses? The curve is still
  climbing and the peak is unbracketed.
- Are split-shift and selective-stash levers ever additive on a less-skewed or
  churn-heavy workload, or is their redundancy trace-specific?
- Untried protected-tier levers beyond capacity and eviction policy (promotion
  position, segment count) remain unexplored.
- All results rest on one skewed trace; does the stash-protection bias
  generalize?

## Per-policy notes

- Recency / promote-on-hit on the protected tier suits the skewed trace well
  and underpins every win.
- LFU eviction on the protected tier is a dead lever: forcing it regressed,
  since LFU retained stale-but-frequent keys over the live working set.
