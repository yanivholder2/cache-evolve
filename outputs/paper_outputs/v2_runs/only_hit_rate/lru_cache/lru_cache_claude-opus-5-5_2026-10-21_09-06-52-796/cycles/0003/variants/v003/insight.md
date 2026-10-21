# Variant v003 Insight

## Helped

(No dimension improved. The larger resident-HIR queue did not produce the hit-rate gain H03 predicted.)

## Hurt

- hit-rate: Moving `hir_cap` from `capacity / 100` to `capacity / 20` takes about 80 slots from the protected LIR set at size 2048 and gives them to the resident-HIR queue. On workload-a, the hot keys lost from LIR slightly outweighed the new keys that the longer HIR residency saved. The regression is small, but it points the opposite way from the hypothesis.

## Takeaway

On this trace, the size of LIRS's resident-HIR buffer is not what holds it back. Keys re-referenced after leaving Q were apparently already being promoted through the non-resident ghost entries, which are bounded at `2 * capacity`, so a bigger resident queue mostly just shrank LIR. This rules out probation-buffer size as the explanation for S3-FIFO's lead, which H03 had set out to test. Further LIRS work should either return to the 1% split or change something other than the HIR/LIR ratio.
