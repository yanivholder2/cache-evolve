# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Halving the sketch's aging threshold made the frequency counts decay sooner, so the admission gate lost popularity history it needed to keep stable victims over newcomers on lirs workload-a.
- metadata-size: The TinyLfuCache struct grew by 16 bytes even though H03 planned no new fields. The change therefore touched more state than the single aging constant, which the hypothesis said it would not do.

## Takeaway

Shortening the aging period on the count-min sketch hurt hit rate on this workload, which suggests stale frequency counts were not the limiting factor for the admission gate. The struct growth and the per-op timing shifts (faster get and put) do not fit a single-constant change, so they are probably measurement noise or side effects of unrecorded edits. The source diff for this variant could not be inspected in this session, so the attribution to the aging constant alone is unverified.
