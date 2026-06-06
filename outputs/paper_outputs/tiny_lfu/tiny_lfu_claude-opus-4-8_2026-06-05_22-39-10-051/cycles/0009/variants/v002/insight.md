# Variant v002 Insight

## Helped

- hit-rate: Scoring the incoming key at the eviction gate by its pre-increment `prior` instead of the post-increment estimate removes the admission self-bias (a once-seen key no longer arrives at `freq = 2` and bumps a frequency-1 resident), sparing a few warm probationary residents that the parent's looser gate displaced — a barely-perceptible uptick over the parent.

## Hurt

- (none observed): Tightening admission so single-occurrence scan keys (prior 0) can never evict and prior-1 keys lose ties did not cost any retained-key hits; the stricter gate only suppressed admissions that were marginal anyway.

## Takeaway

Making the eviction gate self-consistent with the promotion path — both reading the key's history excluding the current touch — is correct in principle but nearly inert on workload-a: the double-counting it fixes only flipped a handful of tie-or-near-tie admission contests, so the hit-rate gain sits at the edge of noise. The change is a cheap correctness/consistency win rather than an exploitable lever here.
