# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Narrowing the sketch counters from `u32` to `u16` is a storage-only change: the hit-rate dimension did not move, which matches the hypothesis's prediction that no counter reaches the saturation point at this size and trace. The footprint benefit (`heap_bytes()` now using `size_of::<u16>()`) is not visible in this fitness vector, which carries only hit-rate, so the metadata-size gain the hypothesis anticipated is unmeasured here and needs a vector that includes that dimension before it can count as a win. The one behavioural difference to keep in mind is that saturation now happens at 65535 rather than `u32::MAX`, so on a longer or more skewed trace the admission comparison could start to tie where the parent still distinguished frequencies.
