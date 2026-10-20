# Variant v002 Insight

## Helped

## Hurt

- hit-rate: H02 changed the admission tie case so that equal sketch estimates admit the candidate and evict the victim. On workload-a at size 2048 this came out slightly lower than the parent, so the claim that admitting ties would help is refuted on this single run. Equal-count cold keys are evidently not a source of recency value that the victim was protecting.
- per-op-time: Put p99 rose relative to the parent. The tie-break only changes which branch runs on equal estimates, so on those puts the code now performs an eviction where it previously rejected the candidate, which is a plausible source of extra put-side tail latency. The get p99 also moved, but the change does not touch the get path, so that movement is more likely noise.

## Takeaway

Breaking frequency ties in favour of the candidate did not raise hit-rate on workload-a, so equal-estimate admissions are not where the parent's admission gate loses value. The change has no metadata cost, and its only measurable effects are a small hit-rate drop and a put tail-latency increase. Two caveats apply: the fitness vector records only one run, and I could not inspect the overlay source from this session (the tools available here could not list or search it), so I have not confirmed that the comparison change landed exactly as the hypothesis describes.
