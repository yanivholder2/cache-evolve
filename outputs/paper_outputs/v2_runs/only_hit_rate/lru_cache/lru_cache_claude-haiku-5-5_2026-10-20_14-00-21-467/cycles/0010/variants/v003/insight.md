# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Cutting the protected segment from 80% to 50% of capacity lets probation hold half the cache, but on workload-a at size 2048 the protected set lost more reused keys than probation gained by absorbing the scan stream, so the 80% share was the better split for this trace.

## Takeaway

Shrinking the protected share from 80% to 50% is a net loss on workload-a at size 2048, which contradicts the hypothesis that the protected share was limiting hits on this trace. The 80% ratio is doing useful work by keeping re-referenced keys resident, and probation absorbing more one-time keys did not compensate. Any further protected-share tuning should be tested on a second trace or size before it is treated as a direction.
