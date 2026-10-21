# Variant v002 Insight

## Helped

- hit-rate: The change drops an incoming key when every slot in its bucket is hot, so a fully reused bucket is not displaced by a key seen only twice. The gain over the parent is within noise, so any benefit is negligible on workload-a.

## Hurt

- None measured. hit-rate is the only reported dimension. The drop path adds a per-eviction check for whether any slot had counter 0, and it forces a third sighting for dropped keys. No dimension captures those costs. I could not inspect the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Blocking admission into an all-hot bucket barely changed the outcome. That suggests such buckets are rare on this trace and size, or that the existing counter decrement and second-sighting filter already protect hot entries well enough. A scan-resistance lever at the bucket level does not look like the bottleneck on workload-a. The result comes from one trace and one size, so scan-heavy phases elsewhere could behave differently.
