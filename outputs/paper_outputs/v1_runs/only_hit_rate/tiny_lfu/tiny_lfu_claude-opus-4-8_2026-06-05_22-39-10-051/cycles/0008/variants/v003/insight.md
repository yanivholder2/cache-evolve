# Variant v003 Insight

## Helped

- (none observed): Aligning the promotion gate's comparison target with its
  actual demotion victim — contesting the promotee against
  `sample_prot_demotee()` rather than the protected LRU head — is logically
  coherent, but it did not raise hit-rate on workload-a at size 2048.

## Hurt

- hit-rate: Relaxing the gate from `> head_freq` to `> demotee_freq` admits
  promotees whose frequency falls between the sampled victim's and the head's;
  on this trace those marginal warming keys displaced protected residents that
  went on to be reused, costing a fraction of a hit rather than gaining one.

## Takeaway

The "head hides a colder neighbour" case the hypothesis targeted does fire, but
swapping in a key that only beats the *sampled coldest* node — not the head —
trades a proven-warm resident for a less-certain promotee, and that gamble lost
slightly here. The stricter head-based gate was not a latent bug to fix but a
conservative guard: contesting against the genuine coldest victim widens
admission into exactly the marginal keys that don't pay off on this workload.
