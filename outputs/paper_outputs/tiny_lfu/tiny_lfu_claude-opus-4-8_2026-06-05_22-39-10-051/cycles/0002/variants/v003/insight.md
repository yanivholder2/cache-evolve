# Variant v003 Insight

## Helped

- hit-rate: Raising `SKETCH_WIDTH_FACTOR` 4→16 quadruples count-min width (8192→32768 at capacity 2048), so fewer distinct keys share cells; the `min`-over-rows estimate overcounts cold keys less, and the admission gate `incoming_freq > victim_freq` now protects fewer cold residents and rejects fewer hot newcomers.

## Hurt

- (none observed): No other fitness dimension is measured; the wider sketch grows memory but that cost is not in the vector, so nothing moved against it.

## Takeaway

On this workload TinyLFU is bottlenecked by estimator noise, not policy design: spending sketch width to sharpen the frequency signal — with the entire LRU/admission path byte-for-byte unchanged — is sufficient to lift hit-rate, confirming the wins come from a more accurate count rather than new mechanics.
