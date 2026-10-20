# Variant v002 Insight

## Helped

- None measured: the only fitness dimension in this variant's vector is hit-rate, and it did not improve.

## Hurt

- hit-rate: New keys no longer face the sketch on arrival. They sit in a 1% admission window (about 20 slots at capacity 2048) and are compared against the main LRU head only when they age out. A key with a short burst of hits inside the window is judged on a sketch count that reflects only that burst, and keys that would previously have been rejected at once now occupy window slots for a while, which costs a little hit-rate on workload-a.

## Takeaway

Putting a small recency window in front of the main LRU, with sketch-based admission applied only at window exit, does not help on workload-a at size 2048 and loses a sliver of hit-rate. The one-touch burst case the hypothesis targeted was not demonstrated on this trace. The change is also structural: the window adds per-node segment state and a second pair of list heads, and this variant's fitness vector includes no metadata or timing measurements, so its cost on those dimensions is unverified.
