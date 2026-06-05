# Variant v001 Insight

## Helped

- hit-rate: Switching the promotion-path insert from `InsertPosition::Front` to `Back` lands a stash hit at its target bucket's MRU end, so a recurring key is immediately the `promotion_candidate_index()` and climbs stash→target→probing in fewer accesses, keeping hot keys resident longer on workload-a.

## Hurt

- (none observed) The single measured dimension moved in the favorable direction; no dimension regressed.

## Takeaway

Back-insertion on the promotion path beats Front on workload-a: surrendering the baseline's one-hit-wonder protection (where a Front-inserted item must re-climb its whole target bucket before promoting) pays off because the trace's genuinely recurring keys reach the probing tier faster than the cost of admitting transient keys one tier deeper.
