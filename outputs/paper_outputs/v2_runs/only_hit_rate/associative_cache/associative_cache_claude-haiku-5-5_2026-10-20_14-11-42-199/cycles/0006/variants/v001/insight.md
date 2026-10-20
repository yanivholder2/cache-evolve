# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Replacing the unbounded `HashMap<K, usize>` frequency counter with a 4096-slot direct-mapped `Vec<u8>` lets unrelated keys share counters, so the admission gate compares frequencies that include other keys' accesses. The `u8` counters also saturate at 255, so hot incoming keys and hot victims tie and the strict `incoming <= victim` check bypasses more inserts than before.

## Takeaway

Bounding the admission gate's frequency metadata cost hit-rate on workload-a, so the hypothesis's "within 0.5 points" bound was not met. The diff changes three things at once (hash-slot aliasing, u8 saturation, and periodic halving every 10 × FREQ_SLOTS accesses), and the single hit-rate measurement cannot separate them. The metadata-size saving the hypothesis aimed for is not in the fitness vector, so it remains unmeasured.
