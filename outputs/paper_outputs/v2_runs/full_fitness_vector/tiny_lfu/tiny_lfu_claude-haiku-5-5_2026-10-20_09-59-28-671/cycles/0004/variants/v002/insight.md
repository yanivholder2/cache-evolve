# Variant v002 Insight

## Helped

- per-op-time: Get latency fell at both p50 and p99, consistent with H02 removing flag-accessor calls from `touch()` and the promotion step, which sit on the get hit path.

## Hurt

- per-op-time: Put latency rose at both p50 and p99, which H02 did not predict; the change touches `unlink()` and promotion, which put can reach on its eviction path, but this is a single run and the put shift may be run-to-run noise.

## Takeaway

Hit-rate and metadata-size held exactly at the parent's values, so the change behaved as a pure access-pattern edit: the packed flag layout and eviction order were preserved, and the timing shift is the only effect. The get improvement supports the claim that the accessor calls were a per-operation cost, but the put regression means inlining is not a clear win on the put side. Confirming the put result needs repeated runs, and the attribution to accessor inlining should be checked against the source diff, which was not inspected in this analysis.
