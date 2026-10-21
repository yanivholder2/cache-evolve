# Variant v002 Insight

(Based on the hypothesis and fitness vectors; the overlay source was not readable in this session.)

## Helped

- hit-rate: Re-admitting a key that matches the last-evicted fingerprint on first sighting lets the filter recover some recently evicted keys that a strict second-hit gate would reject. The effect is negligible, within trace noise.

## Hurt

- per-op-time: Both get and put p50/p99 rose against the parent. The ghost check and ghost write on eviction add work to the admission and eviction path. The get path was meant to be unchanged, so its slowdown suggests extra memory traffic from the shared segment header or measurement noise.

## Takeaway

A one-slot ghost record per segment is nearly free in metadata, which stays at baseline, but it holds only one victim. On a 2048-entry LIRS workload it almost never matches a key that is about to be reused, so it adds latency for essentially no hit-rate gain. Recovering evicted-then-reused keys would need a larger or more targeted ghost history, not a single fingerprint.
