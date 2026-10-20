# Variant v003 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 7 lets a hot main-queue key build up more credit, so it survives more `evict_main` laps than a key reused only once or twice in main. That is the separation the hypothesis was after. The gain is marginal, though. Admission is unchanged (graduation still happens on `freq > 1`), and at cap 3 most hot keys on workload-a were apparently already surviving. A deeper counter adds little beyond that.

## Hurt

- per-op-time (`put`): With a cap of 7, each main eviction has to drain more credit before it finds a `freq == 0` victim. Every unit drained is a `pop_back` + random slab load + `push_front` rotation. Stale keys that once saturated now need up to 7 laps to drain instead of 3. This is the "longer CLOCK sweep" risk the hypothesis named, and it shows up in both `put` p50 and `put` p99.
- per-op-time (`get`, the summary): In the non-test build, `get` is the same code as in the parent: one `index` probe and one saturating increment, with `max_freq()` folding to the constant. The regression therefore does not come from `get`'s own instructions. The most likely cause is the benchmark's mixed get/put stream. Longer `evict_main` sweeps touch many more `slots` and `main` cache lines, which evicts the lines `get` depends on. Some of it may also be run-to-run noise. Either way, this lever is not free on the read path.
- metadata-size: Did not move, as expected. `freq` stays a `u8`, so `Entry` is unchanged.

## Takeaway

The frequency cap is a real but weak hit-rate lever on workload-a. Its cost is paid entirely in eviction work: the drain cost of each main eviction grows with `MAX_FREQ`. So a small hit-rate gain was bought with a large `put` slowdown and a knock-on `get` slowdown. If this lever is revisited, the next step is to read the `h03_split_report` rotations-per-`evict_main` numbers for caps {1, 3, 7, 15}. The question is whether a cap of 4–5, or a cap of 7 with a faster drain (for example halving `freq` on each pass instead of decrementing it), keeps the hot/lukewarm separation without the longer sweep.
