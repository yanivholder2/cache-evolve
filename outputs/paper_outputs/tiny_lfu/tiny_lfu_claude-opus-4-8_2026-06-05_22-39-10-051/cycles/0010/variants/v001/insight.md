# Variant v001 Insight

## Helped

- (none observed): Stacking the `ADMISSION_MARGIN = 1` hysteresis on top of v002's already self-consistent pre-increment score moved hit-rate the wrong way; no dimension improved over the parent.

## Hurt

- hit-rate: Requiring `incoming_freq > victim_freq + 1` instead of `> victim_freq` on top of the pre-increment score double-tightens the gate — newcomers whose prior frequency merely equals or one-ups the victim are now turned away, including warm reuse candidates that v002 correctly admitted, so the cache retains slightly staler residents and dips below the parent.

## Takeaway

The two tightenings do not stack: the pre-increment score had already removed the admission self-bias the margin was meant to counter, so layering a full-count margin on top over-corrects and rejects genuine reuse candidates rather than just suppressing near-tie noise. On workload-a the margin is only safe relative to the *post-increment* score it was designed against (v003); once the score is self-consistent, the extra margin is pure loss.
