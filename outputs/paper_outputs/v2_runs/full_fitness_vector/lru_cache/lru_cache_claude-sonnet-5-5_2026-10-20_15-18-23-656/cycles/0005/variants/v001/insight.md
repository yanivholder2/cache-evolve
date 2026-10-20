# Variant v001 Insight

## Helped

- per-op-time: The fold-free hasher (one multiply plus rotate) shaves a little off the hash step, so get p50 and put p99 moved slightly lower. The shifts are within a nanosecond or a few ns, so they may be measurement noise.

## Hurt

- per-op-time: get p99 rose slightly. The simpler hash may cluster keys a bit more, so some lookups probe longer, but this is within noise and unconfirmed.

## Takeaway

Swapping in the cheaper hasher while leaving the reference-bit logic, eviction walk and node layout untouched kept eviction order identical to v002. Hit-rate and metadata-size therefore did not move. The hash is a small part of per-op cost, so put p50 did not improve and the two parents' gains compose only marginally. Further put-time gains would need to come from somewhere other than the hash function.

Caveat: I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors only.
