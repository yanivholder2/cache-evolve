# Variant v002 Insight

## Helped

- (none) No dimension improved. The only dimension, hit-rate, did not beat the parent.

## Hurt

- hit-rate: It fell marginally below the parent. Doubling the ways per bucket while holding probation at 4 slots makes the protected segment about twice as large, so the protected-MRU demotion path churns less. The wider bucket also pools more keys, which gives scan-like keys more room to displace protected entries.

## Takeaway

Holding probation at an absolute 4 slots on a doubled-associativity bucket did not recover or exceed the parent's hit-rate. The result is essentially neutral, a small regression, so the optimum is not clearly an absolute scan length. Associativity and probation size appear to be weakly coupled on this trace, and the gain is probably not in this knob. This is a null result for the "absolute 4 slots" reading, but it does not fully rule out a proportional optimum, because I could not inspect the code diff.
