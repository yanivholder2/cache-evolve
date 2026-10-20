# Variant v001 Insight

## Helped

- metadata-size: The reuse flag fit into existing bits, so `Item` and the cache struct did not grow. The hypothesis's no-cost-metadata constraint held.

## Hurt

- hit-rate: Gating promotion on a second hit delays the climb of genuinely hot entries. On this trace the first-hit-ignored policy lost more from slow promotion than it gained from keeping one-touch scan entries from climbing.
- per-op-time: Get and put both got slower. The flag test and masking on every key comparison, plus clearing the flag on swap, added work on the hot path. The hypothesis expected the flag to be free because it touches data already loaded, but that did not hold.

## Takeaway

A one-bit "seen once" filter on top of transpose promotion was not a free reuse signal here. It cost both hit rate and latency, so it did not improve on the parent's plain one-step transpose. Transpose is already a gradual, self-filtering promotion, and a second filter in front of it mostly adds delay and per-comparison masking cost.
