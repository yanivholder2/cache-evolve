# Cross-Cycle Insights

## Recurring wins

- Cheap sketch-efficiency levers — reducing hashing cost and narrowing
  counter width — have paid off, buying per-op speed and metadata-size
  savings for only a marginal hit-rate cost.
- Changes that preserve the frequency signal and leave residency/admission
  logic untouched have outperformed changes that alter admission semantics.

## Recurring failures

- Altering the admission signal itself (frequency aging/decay) has not paid
  off: on traces with stable, low true frequencies, decay erases useful mass
  and counters never saturate, so the cost lands without benefit.
- Unamortized inline full-table operations introduce tail-latency variance.

## Open questions

- Is frequency aging a genuine dead end, or just mistuned (decay window too
  tight, sweep unamortized)? Worth retrying with a wide interval and an
  amortized sweep before abandoning.
- How far can counter narrowing go before p99 regresses — does 4-bit packing
  keep the memory win without the tail-latency cost seen at byte width?
- Do sketch-cost changes compose, or do their hit-rate dips compound when
  stacked (e.g. cheaper hashing + narrower counters together)?

## Per-policy notes

- TinyLFU / CountMinSketch admission: the sketch carries exploitable slack —
  speed and memory can be traded out of it cheaply. The recurring soft spot is
  p99/tail latency, surfaced by both inline sweeps and narrow-counter access
  patterns even when p50 improves. Favor signal-preserving, residency-neutral
  reshapings over admission-altering ones.
