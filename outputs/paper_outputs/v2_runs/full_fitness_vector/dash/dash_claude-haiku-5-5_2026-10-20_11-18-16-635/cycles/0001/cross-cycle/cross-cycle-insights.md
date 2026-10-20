## Recurring wins

- Get-path reordering (skipping the stash scan on common target and probing hits) is the cohort's clearest latency lever; it is the only change whose stated mechanism is per-op time. Only one cycle so far, so treat as a lead, not a settled win.
- Insert-position changes move hit rate more than latency. Their effect depends on how full the target buckets are.

## Recurring failures

- Latency gains credited without a matching code-path explanation (including put-side timing movement) have not been confirmed. Do not count them as wins until re-measured.
- Plain MRU promotion on the probing-recovery insert path can displace hot residents of full target buckets, trading hit rate for latency. The one Variant with a recorded hit-rate loss (v001) still scored as a Winner, so the scoring currently favors latency over hit rate.

## Open questions

- Does the get-path latency gain hold on stash hits, once those pay for the two earlier misses, and across reruns?
- Is the v001 hit-rate loss from MRU insertion stable across traces, and does it grow on workloads with fuller target buckets?
- Are the unexplained put-side and per-op movements measurement noise?
- Is weighting latency over hit rate in the Winner criteria intended?

## Per-policy notes

- MRU insertion (probing-recovery path): protects the promoted key from the next eviction, but risks evicting hot target residents under high bucket occupancy. Needs a guard before further work.
- Tier-lookup reordering on the get path (stash scan vs. target and probing scans): not an eviction-policy change; its evidence is latency only, from a single cycle.
