# Variant v002 Insight

## Helped

- hit-rate: Replacing the single strict-LRU head victim with a SAMPLE=5 forward scan that picks the lowest-estimate candidate (ties toward most-LRU) lets eviction skip an old-but-still-warm head node in favor of a genuinely colder neighbor, edging the hit-rate up — though the lift is slim, near the noise floor.

## Hurt

- (none observed): The scan only ranks already-resident nodes and still gates admission with the same `incoming_freq > victim_freq` test, so no candidate that LRU would have kept gets wrongly discarded; no other dimension is measured.

## Takeaway

Layering frequency-ranked victim sampling onto the accurate conservative-update sketch confirms the two levers are independent — victim choice adds a real but tiny gain on top of estimator accuracy rather than being subsumed by it. The payoff is marginal here because the sketch already separates hot from cold well, so a 5-node scan only occasionally finds a better victim than the LRU head; a larger SAMPLE or a more skewed workload is where this lever would matter more.
