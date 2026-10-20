# Variant v001 Insight

## Helped

- per-op-time: Put-path latency moved in the intended direction, with the p50 gain marginal and the p99 gain larger. The hypothesis reuses per-row column indices and skips counter stores that are already at the conservative target, which removes redundant hashing and writes on the put/admission path. The size of the p99 gain is more likely run-to-run variance than a direct effect of this change, since only one run was recorded.

## Hurt

## Takeaway

This variant behaves as a pure access-pattern edit: hit-rate and metadata-size are unchanged from the parent, so the skipped stores did not alter any stored counter or admission decision on workload-a. The put-side timing movement is too small and too lightly sampled to credit to the change. The hypothesis required at least three repeated put measurements, and the record contains one run, so the put result is not yet separated from noise. The overlay source could not be located from this session, so the claim that indices are computed once per key and stores are skipped below target is unverified against the code.
