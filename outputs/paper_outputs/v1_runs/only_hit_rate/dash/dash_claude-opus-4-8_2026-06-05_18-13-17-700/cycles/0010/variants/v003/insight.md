# Variant v003 Insight

## Helped

- (none): no fitness dimension improved over baseline; the doorkeeper's protection of stash residents did not translate into measurable retention of hot keys on this trace.

## Hurt

- hit-rate: gating cold admission on a *second* sighting under stash pressure turns the would-be reuse access into a miss — the keys-only window holds no value, so a key that recurs soon after its first contended sighting is served from nothing and only becomes resident on the access after that, costing hits the baseline's immediate admission captured.

## Takeaway

On this LIRS workload-a trace the contended tail keys recur quickly enough that withholding them on first sighting forfeits a near-term hit instead of filtering a true one-hit-wonder. The doorkeeper protects sitting residents that were not actually under threat from singletons here, so admission filtering is net-negative versus baseline's admit-on-first-touch — the second-sighting tax outweighs the seats it saves.
