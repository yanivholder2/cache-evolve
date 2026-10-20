# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Stacking the lowest-frequency victim choice (buffered over the 8-step second-chance pass in the main queue) on top of v003's 20% small-queue share lowered hit rate below the parent on workload-a at capacity 2048, so the victim choice did not compose with the larger small queue as H01 predicted.

## Takeaway

The victim-choice change does not add hit rate once the small queue already holds 20% of capacity; the parent's partition gain was not additive with it on this trace. The likely reason is that a larger small queue sends fewer entries into the main-queue rotation, so the budget-exhaustion path that the victim choice targets fires less often, and the frequency-based pick may then evict entries that tail-drop would have kept. I could not read the overlay source in this session, so this mechanism is unverified. The get and put latency movements are not attributable to this change, since the hypothesis left the get path alone and the put shift is small.
