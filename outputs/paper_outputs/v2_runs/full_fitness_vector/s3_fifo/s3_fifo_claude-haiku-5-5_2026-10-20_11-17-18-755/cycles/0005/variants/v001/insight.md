# Variant v001 Insight

## Helped

- hit-rate: Capping the ghost population at half the main queue (`ghost_cap`) means keys evicted longer ago are forgotten sooner, so fewer of them are readmitted straight to main on a later put. The gain is small and comes from one trace at one capacity, so treat it as a directional signal.
- per-op-time: Put-path latency fell against the parent. The likely cause is that ghost overflow handling now triggers at the smaller `ghost_cap`, so the ghost set and `ghost_copies` map stay smaller and the hash work per put drops. The explicit front-drain before `ghost.push_back` keeps stale copies from piling up in the queue.

## Hurt

- metadata-size: The new `ghost_cap: usize` field adds one word to the `S3Fifo` struct. `Entry` is unchanged, so the cost sits in the container, as the hypothesis anticipated.
- per-op-time: The summary dimension is get p50, and it moved up even though `get` and the frequency logic are byte-for-byte the same as the parent. Nothing in this change touches the get path, so the regression is more likely measurement noise or code layout than a real cost of the change. It is recorded here because the summary moved, not because the change caused it.

## Takeaway

Halving the ghost cap and bounding the physical ghost queue trades a little ghost memory for a small hit-rate gain and a cheaper put path on workload-a at 2048. That suggests the parent's ghost population was longer than this trace needs. The gain rests on one trace and one capacity, and the get-side movement is unexplained by the diff, so this is a lead to confirm on other traces and sizes rather than a settled win.
