# Variant v003 Insight

## Helped

- hit-rate: Capping the fingerprint ghost at half the main queue's capacity makes the ghost forget older fingerprints sooner, so keys that return only after a long gap are less likely to be admitted to main, which keeps main filled with keys that are actually reused on the measured trace.

## Hurt

- metadata-size: The `S3Fifo` container grew by a small amount, which fits a single stored `main_capacity` field that the hypothesis computes once at construction; the entry layout itself is unchanged.
- per-op-time: Put is measurably slower, plausibly because a full ghost now runs the shared expiry routine on inserts to enforce the new cap. Get should not be affected by a ghost-only change, so the get slowdown is more likely measurement noise or code layout than a real effect of this diff.

## Takeaway

Shrinking the ghost trades a small hit-rate gain for extra work on the insert path and a few bytes of container state, so the change only pays off if the gain holds across traces. The source diff could not be inspected in this run (the overlay files were not reachable from this session), so the attribution above rests on the hypothesis text and the fitness deltas; confirm the `main_capacity` field and the expiry call site against the overlay before relying on the metadata and put-latency explanations.
