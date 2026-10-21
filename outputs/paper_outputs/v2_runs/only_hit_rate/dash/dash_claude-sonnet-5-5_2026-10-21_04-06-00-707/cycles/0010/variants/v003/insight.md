# Variant v003 Insight

## Helped

- (none) No dimension improved. The adaptive second-sighting gate and the reuse-flag victim preference did not beat the baseline on any measured dimension.

## Hurt

- hit-rate: Hit-rate on the single lirs workload-a trace (size 2048) came out slightly below the baseline. The never-hit/hit eviction ratio over a 4096-eviction window is a lagging signal. When the gate flips to strict mode, it rejects first-time keys that would have been re-referenced soon. The 1-bit filter has no aging, so stale bits add false admissions or rejections.
- metadata-size, per-op-time: The fitness vector has no entries for these dimensions, so this variant's cost can't be compared with the baseline. The change adds a 8x-slot-count bit array, a per-slot reuse flag and window counters. It also puts extra work on the miss and eviction paths. These are unmeasured costs, not demonstrated ones.

## Takeaway

Making admission adaptive added moving parts (window counters, a mode switch, an unaged bit filter, reuse flags) and did not recover the hit-rate of the plain baseline policy. The never-hit-eviction ratio is too noisy and delayed a trigger to switch strictness on a trace like this one. Any future admission-filter attempt should include its own aging or a faster signal. It should also be evaluated on all dimensions, since this run only reported hit-rate.
