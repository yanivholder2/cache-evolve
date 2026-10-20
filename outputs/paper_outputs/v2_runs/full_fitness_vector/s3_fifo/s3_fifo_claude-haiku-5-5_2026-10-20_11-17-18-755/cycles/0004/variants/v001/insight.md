# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Sizing the ghost queue to the full cache capacity instead of the main-queue size let more evicted keys stay remembered, and each ghost hit now routes a key straight into main; on workload-a at capacity 2048 that admitted keys whose reuse did not follow, displacing resident entries and lowering the hit rate below v003's.
- per-op-time: The change touches only the ghost capacity constant, so the get path is unaffected in principle; the put path is where ghost membership is checked and ghost hits promote into main, so the larger ghost likely added put-side work. The get-side movement is not explained by this change and is more plausibly measurement noise.

## Takeaway

Enlarging the ghost queue from main-sized to full-capacity is a regression on this trace: the extra remembered keys are re-admitted to main without a corresponding reuse gain, so the ghost capacity is not a free dial in this design. metadata-size is unchanged as the hypothesis predicted, and the hidden ghost-key memory cost the hypothesis accepted was not measured here. I could not read the overlay source in this session, so the mechanism above is inferred from the hypothesis text and the fitness deltas rather than confirmed against the diff.
