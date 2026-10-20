# Variant v003 Insight

## Helped

- (none observed): Shrinking the protected tier to two-thirds of capacity did not raise hit-rate; the enlarged probationary buffer gave the admission gate no measurable extra screening benefit.

## Hurt

- hit-rate: Lowering `protected_cap` from `(capacity*4)/5` to `(capacity*2)/3` slightly reduced hit-rate — the smaller protected tier evicts proven-hot promoted keys sooner, so a few residents the larger tier had sheltered now get demoted toward the probationary victim end and lost.

## Takeaway

On workload-a at size 2048, the SLRU split favours a *larger* protected tier, not a smaller one: enlarging probation to two-thirds traded away protection without buying admission accuracy, since the frequency sketch already screens entrants. This answers cycle 0004's open question — the protected fraction was not under-tuned downward; the marginal SLRU effect is a workload ceiling, and reducing the protected cap only erodes it.
