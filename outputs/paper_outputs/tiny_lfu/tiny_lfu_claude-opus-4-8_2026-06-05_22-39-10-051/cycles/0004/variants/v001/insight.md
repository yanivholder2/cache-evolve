# Variant v001 Insight

## Helped

- (none): Stacking the factor-64 sketch on top of v002's conservative-update `increment` produced no measurable hit-rate change — the combined estimate did not gate any admission more accurately than either lever alone.

## Hurt

- (none): Widening from factor 16 to 64 with CU preserved kept every admission decision identical; nothing regressed and no other dimension is measured.

## Takeaway

Running width×CU *together* in one variant lands on the same hit-rate that the wide-sketch winner and the CU winner each reached separately, confirming the cycle-0003 redundancy conclusion that was previously only inferred. At factor-64 width, cell collisions are already so rare that conservative update has no residual overcounting left to correct, so the two accuracy levers neither stack nor interfere on this workload.
