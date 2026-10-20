# Variant v003 Insight

## Helped

- hit-rate: The residency-class tie-break (prefer evicting a stash resident over a target-or-probe resident when counters tie) moved hit-rate in the hypothesis's predicted direction, but the margin is too small on a single trace to separate from noise, so the tie-break is at best a weak lever on this workload.

## Hurt

- per-op-time: Get latency rose clearly relative to the parent even though the hypothesis only touched victim selection in the eviction scan, which runs on the put path; a get-path slowdown is not explained by the described change and points to either an unintended edit to the get path or measurement variance.
- per-op-time: Put latency also rose, which matches the hypothesis's warning that the extra residency comparison in the single-pass scan would add cost, though the increase is larger than one extra comparison per slot would plausibly produce.
- metadata-size: No change, as expected, since the record layout was preserved.

## Takeaway

Breaking counter ties by residency class rather than by age produced at most a marginal hit-rate gain on workload-a while adding measurable latency on both get and put, so residency is not a strong tie-break key for this policy. The get-path regression is the open question: the hypothesis did not touch that path, and the overlay source could not be inspected in this session, so the cause (overlay edit vs. noise) remains unconfirmed and should be checked before reading this variant's cost as a property of the tie-break itself.
