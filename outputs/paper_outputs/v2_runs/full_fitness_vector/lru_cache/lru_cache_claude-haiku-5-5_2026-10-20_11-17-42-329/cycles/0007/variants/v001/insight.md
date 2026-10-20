# Variant v001 Insight

## Helped

## Hurt

- per-op-time: Folding the high half of the FastHasher state into the low bits in `finish()` adds a shift and xor to every hashed lookup, and the get and put paths got slower on both median and tail, so the extra mixing cost more than any bucket-spread benefit it bought.

## Takeaway

The fold did not help: the parent's raw `finish()` already spreads bucket-index bits well enough for workload-a's integer keys, so the hypothesis's premise that the low bits were under-mixed does not hold here. Hit-rate and metadata-size stayed exactly flat, as expected from a change that touches only the hashing path, so the regression is purely the cost of the extra finish-step operations.
