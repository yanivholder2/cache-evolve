# Cross-Cycle Insights

## Recurring wins

- Cost/layout levers that leave admission semantics untouched are reliable,
  near-free wins: narrowing counters to `u8`, swapping in a cheap
  multiply-xorshift hasher, and re-encoding intrusive links as `u32` sentinels
  instead of `Option<usize>` all landed clean across cycles 0001–0004.
- Composition of proven efficiency/layout winners is additive, confirmed
  repeatedly (0003, 0004, 0005): changes touch disjoint structures (sketch heap
  vs. node layout vs. hasher), so gains stack without their downsides stacking.
- Sharpening the existing frequency signal beats adding admission *structure*.
  Widening the sketch to cut collisions delivered the strongest hit-rate lifts
  (0004–0005); early on these were free, but by 0005 every signal knob carries
  an axis-local price.

## Recurring failures

- Counter aging/decay is a settled dead end on this trace: lost in 0001, retried
  wide and amortized in 0007, still a net loss. The hot set is stable and
  low-frequency, so decay erases earned mass faster than it refreshes contrast
  while taxing every increment.
- Structural admission machinery underperforms: a ~1% W-TinyLFU recency window
  (0004) and a one-count admission margin (0008) were inert — too small to buffer
  newcomers or flip decisions on a hot set whose frequencies are widely separated.
- Buying back row-hash independence (real second hash, 0006) yields no accuracy
  gain — row correlation was never the bottleneck — and only slows ops.
- Re-shaping sketch memory (flattening `Vec<Vec<u8>>` to one buffer, 0006) does
  not recover the wide sketch's per-op tax; the cost is footprint/cache-line
  stride, a real floor, not a layout artifact.

## Open questions

- Is hit-rate effectively saturated at the incumbent wide-sketch base? All three
  named hit-rate surfaces (estimate rule, admission margin, sketch width) were
  ruled flat in 0008.
- Is there any genuinely new admission/eviction *structure* (not a constant or
  margin tweak) that moves hit-rate, or should the frontier shift to banking
  cost-axis savings (footprint, p99 tail) at fixed hit-rate?
- Does conservative update's narrow-sketch (factor-4) gain ever justify its
  hot-path cost, given it does not transfer to the wide (factor-16) base?

## Per-policy notes

- Sketch / admission (TinyLFU CountMin): the estimate's sharpness — not row
  count, sample depth, or hash independence — is the binding constraint on
  quality. Conservative update helped only on the narrow sketch; on the wide
  sketch the aliasing it corrects is already suppressed, so width, depth, and
  precision knobs are accuracy-saturated and now only buy footprint or tail.
- Eviction (victim selection): sampled-coldest victim walks lift hit-rate only
  marginally and originally cost p99 tail; moving the walk onto a single cheap
  hash (0003) turned that tail cost net-positive. Victim-sample depth is
  accuracy-saturated (0005, 0007) — tune it for tail, not hit-rate.
- Tail latency is a persistent soft spot across policies: inline sweeps, narrow
  counter access, and deeper eviction sampling each nudge p99 the wrong way even
  when p50 improves.
