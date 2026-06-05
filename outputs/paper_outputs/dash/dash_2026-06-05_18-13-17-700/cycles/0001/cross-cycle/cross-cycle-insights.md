## Recurring wins

- Protecting the warm working set in the normal tier is the dominant winning
  lever: budget conserved, invariants preserved, hit-rate up with no observed
  regression on other dimensions.
- Reallocating fixed budget toward the promotion-protected normal tier
  out-performed admission-side tweaks aimed at the same imbalance.

## Recurring failures

- No approach has repeatedly failed yet; single cohort observed. Stash-admission
  tweaks (back-insertion, frequency gate) helped but did not top capacity
  reallocation — a lead to watch, not yet a confirmed failure.

## Open questions

- Are the stash-protection levers (capacity split, insertion position,
  promotion gate) additive or redundant when stacked, since all attack the
  same stash-vs-normal imbalance?
- The winning split ratio and promotion threshold are single unswept points,
  not optima.
- Does the stash-protection bias generalize beyond one skewed trace to less
  skewed or churn-heavy workloads?

## Per-policy notes

- DASH tiering: baseline appears to over-provision cold stash admission;
  both shrinking it and making it more selective improved hit-rate, pointing
  to one underlying weakness rather than independent ones.
