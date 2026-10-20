# Variant v003 Insight

## Helped

- hit-rate: Remembering probationary-evicted keys in the bounded ghost FIFO and re-admitting a returning key at the protected LRU end (`push_front_protected`) recovers a few reuse-distance hits — keys that fell just outside the probationary window now survive one extra probationary eviction instead of starting over cold.

## Hurt

## Takeaway

Weak protection — entering ghost hits at the protected LRU end and immediately running `enforce_protected_budget` — recovers reuse-distance signal without crowding the hot set, but on this trace the effect is barely above noise: most reuse is already captured by the second-access promotion the parent SLRU provides, so the ghost record only rescues the thin band of keys whose reuse distance lands just past probationary capacity. This corrects cycle 0002's protected-MRU placement (which crowded hot keys) and confirms the placement was the problem, but the recoverable headroom here is small.
