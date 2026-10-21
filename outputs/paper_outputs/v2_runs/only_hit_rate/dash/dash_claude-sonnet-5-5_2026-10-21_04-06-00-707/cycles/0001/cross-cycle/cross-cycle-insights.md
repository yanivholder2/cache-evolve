## Recurring wins

- Changing *where* a promoted item lands (MRU insertion) beat changing *how many* items move. Only one cycle of evidence so far; treat as a lead, not a settled pattern.
- Recency-skew rationale (don't make just-hit items the next victim) held up.

## Recurring failures

- Adding movement (more promotions, bidirectional probing<->target swaps) gained little or lost. Likely cause: demotion cascades toward stash/eviction offset the benefit.
- Relaxing promotion eligibility alone gave only marginal gain; spare probing capacity is not a binding constraint.

## Open questions

- Does MRU placement generalize beyond workload-a, and does it cost per-op time?
- Does combining MRU insertion with eligibility relaxation add anything, or was the marginal gain noise?
- Do insertion-position changes on other paths (demotion, cascade to stash, stash->target vs. probing retries) help?
- Metadata size and per-op time have never been measured; cost claims are impossible until they are in the fitness vector.

## Per-policy notes

- Dash segment promotion (`DashSegment::get`/`promotion_get`): the insertion-position lever is the productive direction; movement-count levers are not. Baseline comparisons so far cover hit-rate only.
