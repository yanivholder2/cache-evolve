# Variant v002 Insight

## Helped

- hit-rate: Requiring the incoming key to beat the victim by `ADMISSION_MARGIN = 1` filtered a few transient mid-frequency newcomers from displacing settled incumbents, nudging hit-rate up by a hair over the parent.

## Hurt

- per-op-time: The measured `get`/`put` latencies rose, but the only code change is one extra integer add inside the already-full-only eviction branch in `put` — it cannot touch the `get` path at all, so this swing is benchmark noise rather than a cost of the margin.

## Takeaway

Admission hysteresis with a single-count margin is essentially neutral on this trace: the stable hot set's frequencies are already separated from cold-victim estimates by far more than one count, so the +1 threshold rarely flips an admission decision either way. The cheapest possible hit-rate lever barely moves the needle here, which says the parent's admission test was not the bottleneck — the limiting factor is frequency *estimation/geometry*, not the admission *margin*.
