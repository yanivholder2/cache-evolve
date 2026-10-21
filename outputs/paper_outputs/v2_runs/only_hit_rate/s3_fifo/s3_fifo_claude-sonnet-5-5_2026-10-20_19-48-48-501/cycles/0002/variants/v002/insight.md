# Variant v002 Insight

## Helped

- None: no measured dimension improved over the parent.

## Hurt

- hit-rate: Letting the ghost queue hold 2×main_size keys makes ghost hits less selective. More small-queue one-hit wonders and main victims stay "remembered", so their next access goes straight into main with freq 0. That pollutes main and displaces residents that would have been reused.
- metadata-size: Not measured in this evaluation. The ghost queue can now hold twice as many keys, so this is expected to cost more memory, but it is unverified.

## Takeaway

On this workload, a longer ghost memory is not what limits S3-FIFO. Reuse distances beyond the ghost's old window are rare, and the extra ghost hits mostly admit weakly-reused keys into main. The ghost length acts as an admission filter, so lengthening it loosens that filter. Any gain would need a stricter ghost-hit admission rule alongside it.
