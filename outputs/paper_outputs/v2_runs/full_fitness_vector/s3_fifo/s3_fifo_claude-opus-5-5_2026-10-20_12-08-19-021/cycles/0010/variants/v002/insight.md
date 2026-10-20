# Variant v002 Insight

## Helped

- (none observed) hit-rate: The hypothesis predicted a gain, and there was none; hit-rate is identical to the parent. Clearing the freq bits (`*e &= SLOT_MASK`) in the graduation branch of `evict_small_inner` removes the 2–3 free CLOCK laps a graduate used to bring into main. On workload-a at size 2048, that never changed which key `evict_main` releases. Graduates that are actually hot get re-hit in main and climb back to `MAX_FREQ` before they reach the tail. The leftover credit only delayed graduates that were never touched again, and the delay was too short to change any eviction decision. The hit-rate did not fall below the hypothesis's threshold either, so the counter-claim fails too: early hits in small neither help nor hurt residency in main here.

## Hurt

- per-op-time (`get`, the summary): `get` got slower even though `get` is byte-identical to the parent: one `index.get_mut`, a saturating freq bump, and one slab load. The diff never touches it, and the identical hit-rate means the hit/miss mix is the same. So the regression can't be caused by this change. It is most likely benchmark drift, which the parent's insight had already warned about.
- per-op-time (`put`): `put` also got slower, by a similar ratio to `get`. The change only adds one store to the `u32` it already probed for the graduation test, and it should shorten `evict_main` drains, not lengthen them. When both ops slow down together and the code paths didn't change, that points to the machine, not the code. A side-by-side re-run against the parent is needed before anyone credits or blames freq-reset for any latency change.
- metadata-size: unchanged. The reset reuses the existing packed `(freq << 30) | slot` word, so it adds no fields or bytes.

## Takeaway

Under S3-FIFO with `MAX_FREQ = 3` and decrement drain, a graduate's inherited small-queue credit makes no difference to hit-rate on workload-a. Resetting it at graduation is a no-op for hit-rate, and you get the original S3-FIFO behaviour (main residency earned only by hits in main) at no cost. Keep it if the simpler semantics are wanted, but it is not a hit-rate lever. Future hit-rate gains will have to come from elsewhere, for example the graduation threshold or the split, not from how much credit a graduate carries. I couldn't confirm the expected drop in `main_rotations` from the eval artifacts. Check it in the `h01_split_report` output, because the latency numbers in this run can't show it.
