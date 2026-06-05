# Variant v002 Insight

## Helped

- hit-rate: Replacing strict probationary-LRU eviction with an 8-node frequency-aware scan (saturating `freq` per node, ties broken LRU-most) spares warm probationary keys that a scan flood would otherwise push off the head, edging the reuse-heavy LIRS trace up.

## Hurt

## Takeaway

On this trace the frequency-windowed victim choice barely beats the v001 SLRU split: the protected tier already absorbs most of the reuse signal, so the bounded probationary scan only rescues the narrow set of warm-but-unpromoted keys, leaving hit-rate essentially flat. The added per-node counter, aging sweep, and window walk buy almost nothing here because two-access promotion is doing the heavy lifting, not probationary eviction order.
