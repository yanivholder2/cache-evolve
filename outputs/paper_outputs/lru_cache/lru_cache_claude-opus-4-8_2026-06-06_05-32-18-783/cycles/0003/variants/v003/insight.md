# Variant v003 Insight

## Helped

- per-op-time: the `get` MRU fast path (`tail == link(index)` early return) shaves the get p50 by skipping the unlink/push_back pair on the recurring hot key that already sits at the tail, nudging the summary latency down a hair.

## Hurt

- per-op-time: `put` p50 and p99 both regressed because its `tail == link(index)` guard sits on the existing-key update path where the touched node is usually *not* the tail, so the extra compare runs and falls through to the same surgery it was meant to skip — pure added cost.
- per-op-time: `get` p99 also rose, so the added branch on the universal hot path lengthens the tail rather than only trimming the median.

## Takeaway

Grafting v002's order-preserving short-circuit onto single-segment plain LRU does not reproduce its segment-local win: on this trace the touched node is rarely already the MRU, so the `tail == idx` check behaves like cycle 0002's bare-branch misprediction case — costing put and tail latency for a ~1 ns get-median gain and staying nowhere near v003's 39. The position gate only pays off when the predicted position is actually common, which it is not at the head of a one-list LRU.
