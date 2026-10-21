## Recurring wins

- (Only one cycle so far, so nothing has recurred yet. The items below are tentative.)
- Mechanical speedups that leave the policy alone, like v003's single fast hash and flat sketch, have been the safest path to a Winner. The hit-rate change was small and is probably a side effect of a different collision pattern.
- Keeping long-run frequency history has beaten freshness on workload-a. Any change that keeps the baseline's accumulated counts has held up.

## Recurring failures

- Restructuring the resident layout with a tiny window: a 1% window at size 2048 is only about 20 slots. The admission contest decides the outcome anyway, so changing the layout alone does nothing.
- Aggressive aging (halving at a 10×capacity sample) combined with conservative update: it erases useful history. Coarse counts then tie under the strict `>` comparison, so victims stay put and hit-rate falls.
- Evaluations that report only hit-rate: latency and metadata-size claims stay unverified, so speed-oriented hypotheses can't be confirmed.

## Open questions

- Is the bottleneck in the admission contest itself, meaning how estimates are compared and how ties are broken? Possible tests: a tie-breaking rule, a doorkeeper, or a randomized admission rule.
- Can aging help when tested on its own, without conservative update and with a much larger sample period? v002 changed both at once, so neither effect is known separately.
- Would a larger or adaptive (hill-climbing) window let a segmented layout pay off, especially combined with gentler aging?
- Does the fast-hash change actually lower get/put latency and metadata-size? This needs a benchmark that reports per-op-time.

## Per-policy notes

- TinyLFU, baseline/admission: the frequency contest is the main lever for hit-rate. With low counts it often ties, and ties favor the incumbent victim.
- TinyLFU, sketch: switching to a single fast hash is neutral on policy and a sound base to build on. Halving plus conservative update hurt workload-a.
- W-TinyLFU, window/SLRU: at a 1% window it behaves almost exactly like the baseline. It is untested at larger or adaptive window sizes.
