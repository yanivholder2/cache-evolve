# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Capping the frequency counter at MAX_FREQ=8 had no effect on this workload: hit-rate was unchanged from the parent. The demotion halving already in place (`freq /= 2` on every protected-to-probationary transition) prevents counter runaway in practice — an entry must stay in the protected segment continuously, without ever being demoted, for its counter to inflate far above 8. On workload-a that residency pattern does not occur at cache size 2048, so the cap is never the binding constraint and contributes nothing. The hypothesis is structurally correct but targets a pathology this trace does not exhibit.
