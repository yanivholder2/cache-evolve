# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Raising `protected_cap` from 95% to 97% shrank the probationary segment from ~102 to ~61 slots at capacity 2048, narrowing the reuse window that catches second touches and costing hits.

## Takeaway

Pushing the SLRU split past 95/5 to 97/3 regressed hit-rate, resolving the open geometry question in the negative: 95/5 is the converged ceiling, not a step short of it. The marginal protected slots gained from 95→97% are worth less than the probationary reuse window they consume, so the probationary floor's size — not further protected residency — is the binding constraint. The protected-ratio axis can be closed at 95/5.
