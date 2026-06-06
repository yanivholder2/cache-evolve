# Variant v003 Insight

## Helped

- (none observed): The admission-only doorkeeper — gating eviction-path arrivals on a second sighting within a `capacity*8` window, leaving the count-min sketch untouched — did not raise hit-rate; it landed essentially flat against the parent.

## Hurt

- hit-rate: Requiring a churn key to appear twice on the eviction path before it can contend cost a hair of hit-rate rather than gaining one — the doorkeeper occasionally rejects a borderline arrival that the bare `incoming_freq > victim_freq` test would have admitted, with no compensating gain from blocking one-hit-wonders.

## Takeaway

On this workload the frequency comparison already screens one-hit-wonders effectively, so layering an admission-only doorkeeper in front of it adds a redundant gate that contributes nothing and slightly delays legitimate warming keys. The parked "long-window doorkeeper" mechanism, even isolated on the wider sketch and never touching the main counters, is not the lever here — estimator accuracy, not extra admission filtering, is what governs hit-rate.
