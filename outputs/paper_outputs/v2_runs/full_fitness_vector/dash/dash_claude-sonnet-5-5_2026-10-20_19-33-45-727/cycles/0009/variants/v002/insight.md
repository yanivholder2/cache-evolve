# Variant v002 Insight

Note: the shell and search tools were unavailable, so I could not diff the overlay source. This insight rests on the hypothesis and the two fitness vectors.

## Helped

- metadata-size: No change. The doorkeeper lives outside the per-item struct, so removing a probe did not touch the item layout.

## Hurt

- per-op-time: Put latency got worse, the opposite of the goal. Dropping the second probe saved little work because both probes already sat in the same cache line. The higher aging threshold (about 37% fill) probably made resets fire at a different cadence, and the lost second-probe selectivity may have changed how often puts take the slower admit path.
- per-op-time: Get latency also got worse even though the get path was untouched. This points to indirect effects such as more admitted keys and extra eviction churn, or to measurement noise, rather than the probe change itself.
- hit-rate: Hit-rate dipped slightly. A single bit probe has a higher false-positive rate than two bits, so more one-hit keys passed the doorkeeper and polluted the cache. Raising the threshold did not fully offset this.

## Takeaway

Once both probes fall in one cache line, the second probe costs almost nothing, so removing it saves no time. It does weaken the filter's discrimination. Further put-latency gains need to come from somewhere other than probe count, such as reset cost or lock hold time.
