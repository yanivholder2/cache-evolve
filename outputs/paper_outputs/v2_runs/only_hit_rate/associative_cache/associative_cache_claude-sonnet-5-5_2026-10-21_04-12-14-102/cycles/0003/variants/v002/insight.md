# Variant v002 Insight

## Helped

- None observed. Only hit-rate was measured, and it did not improve over the parent.

## Hurt

- hit-rate: The hypothesis added a per-bucket ghost ring of recently evicted probationary keys and sent ghost hits straight to the protected segment. On this LIRS trace that appears to promote keys too eagerly. Keys that were evicted once and seen again are not necessarily hot. They displace protected entries that had already earned their place, which weakens the scan resistance v001 had gained.

## Takeaway

A 2Q-style A1out ghost history did not help on top of v001's probation/protected split. A ghost hit is weaker evidence of reuse than a second hit on a resident entry. With 16 entries per 64-way bucket, the ring is also likely too small or too noisy to separate real reuse from scan repeats. A tighter promotion rule would be a better next step, such as requiring a ghost hit plus another signal, or a smaller protected-victim penalty. Also unmeasured are per-op-time and metadata cost from the added ring.
