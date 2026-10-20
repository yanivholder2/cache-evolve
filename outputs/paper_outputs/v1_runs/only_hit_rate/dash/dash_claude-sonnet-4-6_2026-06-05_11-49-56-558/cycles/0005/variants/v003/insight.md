# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Inserting newly-promoted items at `InsertPosition::Front` (LRU/probationary end) of the probing bucket instead of `Back` (MRU) causes recently-graduated items to evict from probing sooner than they can earn a re-access, slightly increasing the rate at which hot items are demoted before being re-hit.

## Takeaway

The probationary-zone idea relies on a population of transient one-off promotions to filter; on workload-a, items that reach the hottest position in their target bucket appear to be genuinely reused, so Front-insert in probing removes MRU protection from items that would have benefited from it rather than weeding out noise. The stash→target Back-insert gain inherited from v002 is preserved, but the new Front-insert at the probing tier works against it by shortening the effective dwell time of just-promoted items — suggesting that MRU re-entry is valuable at both promotion steps on this workload, not just the stash→target one.
