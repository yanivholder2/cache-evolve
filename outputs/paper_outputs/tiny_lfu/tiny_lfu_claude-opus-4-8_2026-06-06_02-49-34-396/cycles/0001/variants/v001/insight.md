# Variant v001 Insight

## Helped

- metadata-size: the aging fields (`size: u64`, `sample_size: usize`) live on the
  `CountMinSketch`/cache struct, not on `Node`, so the per-entry summary metric was
  untouched — aging buys adaptivity without growing per-key overhead.

## Hurt

- hit-rate: the hypothesis predicted aging would *raise* hit-rate, but it fell. With
  `sample_size ≈ 10 × capacity`, `reset()` halves every counter far too often for this
  trace, erasing accumulated frequency mass so the `incoming_freq > victim_freq` test
  admits marginal newcomers and lets genuinely hot keys get evicted.
- per-op-time: `reset()` sweeps all `depth × width` counters inline inside `increment()`,
  so the put that crosses the sample threshold pays an O(width) burst — visible as a
  worse put p99 tail even though common-path p50 didn't regress.

## Takeaway

Frequency aging is real TinyLFU, but a decay window pinned at ~10× capacity is too
aggressive here: it throws away the very frequency signal the admission test depends on,
so aging hurt hit-rate instead of helping it. The mechanism isn't wrong — the sample
size is mistuned, and the inline full-table sweep also injects a put tail-latency spike.
Future variants should widen the decay interval (or amortize the sweep) before
concluding aging is a dead end.
