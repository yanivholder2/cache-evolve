# Variant v002 Insight

## Helped

- per-op-time: Put's median time dropped against the parent, consistent with the change that replaces the per-slot clock-walk loop with a single rotate-and-trailing-zeros search plus one mask clear, which is the put-side scan cost H02 targeted.
- per-op-time: Get's median and tail both dropped even though H02 leaves the get path (one bucket-word OR) untouched, so this gain is not explained by the scan change and should be treated as likely run-to-run variance or code-layout effects.

## Hurt

- per-op-time: Put's p99 rose slightly while its p50 fell, so the word-level scan removed typical eviction cost but did not reduce the tail.

## Takeaway

Hit-rate and metadata-size stayed exactly where v001 left them, which is what the stated invariant (identical victim selection and mask state) predicts, and the put median improvement supports the claim that bit-by-bit scanning caused part of v001's put regression. The get improvement is unexplained by this diff, and the overlay source could not be read in this session, so the mechanism is not confirmed from code.
