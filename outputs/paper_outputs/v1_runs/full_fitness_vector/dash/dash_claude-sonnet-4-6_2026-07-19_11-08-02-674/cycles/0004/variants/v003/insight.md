# Variant v003 Insight

## Helped

*(no dimensions moved positively)*

## Hurt

- **metadata-size**: `Item<u64,u64>` grew by 16 bytes relative to the parent, but the overlay only adds a runtime counter reset (`data.lfu_counter = 0`) with no structural changes to `Item` — the size increase reflects baseline evolution between cycles, not the counter-reset itself.

## Takeaway

Resetting `lfu_counter` to zero when an item is promoted from stash to target (to prevent stale pre-demotion counts from prematurely re-triggering the vacancy gate) produced no measurable change in hit-rate on workload-a, suggesting that counter-carrying oscillation between stash and probing is not a frequent enough pattern in this workload at this cache size to be detectable. The two new unit tests confirm the mechanism is structurally correct, but the workload trace simply does not exercise the re-entry oscillation scenario often enough for the guard to translate into observable hit-rate gains.
