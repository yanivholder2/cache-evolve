# Variant v002 Insight

## Helped

- hit-rate: Ghost hits older than half the ghost ring are now treated as fresh misses and go to small, not main. This keeps stale re-admits out of main. The gain is real but very small.

## Hurt

- per-op-time: Both get and put slowed, with the put tail slowed most. The change adds a monotonic sequence counter, a sequence field in the ghost map value, and a ring-distance computation on every put that checks the ghost. The `get` slowdown is not explained by that logic, because `get` should not touch the ghost path. It more likely comes from a larger map value or from measurement noise.

## Neutral

- metadata-size: Nothing moved. The reported struct sizes did not change. The sequence field may be absorbed by padding or sit in heap-allocated ghost entries that the measurement does not count.

## Takeaway

Age-gating ghost promotion to the newest half of the ring barely changes hit-rate on this trace, so stale ghost re-admits were not a significant source of weak main entries. The extra bookkeeping cost clearly more latency than it gained in hit-rate. Note: I could not read the overlay sources in this session, so this analysis rests on the hypothesis and the fitness vectors.
