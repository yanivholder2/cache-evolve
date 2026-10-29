# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0005 v003 (coldest-of-4 sampled victim, double counting kept). Each changed only the sketch: periodic halving every 10×capacity gets (v001), conservative update (v002), and hashing once with double hashing in a flat array, which is a latency-only change (v003). v002 was named winner, but only relative to the cohort. It lifted hm_0 small and lost web_0 at both sizes. No variant improved on the parent, and every variant hurt web_0.

## Patterns

- In this lineage the sketch is not a measurement tool. It is the admission policy itself. Each variant changed the sketch's error profile, and each change moved the admission bar in a way nobody intended. Halving zeroes the cold probation tail, so newcomers clear the bar too easily. Conservative update produces exact low-end ties, which raise the bar under strict `>`. Correlated double-hash collisions tip close contests. The parent's web_0 gain depends on a finely balanced mix of double-count inflation, random collision noise and strict `>`. Any change to one of these pieces upsets the balance.
- Ties and near-ties at low counts decide outcomes, not collision inflation. The sketch is wide enough that inflation of the minimum is rare. Most decisions are made at the low end of the count range.
- Aging on its own does not add recency. It amplifies the double-count newcomer bias. This refutes the cycle 0006 claim that "the unaged sketch is the root cause" in isolation. Aging and the double count must change together.
- hm_0 small moved slightly up in all three variants, which is the first time in several cycles. Stickier incumbents (v002) helped it most. This hints that hm_0 small wants fewer false admissions, not more recency.
- Row independence in the sketch carries real accuracy on web_0 large. Cheaper hashing is not free.
- Per-op-time still cannot be scored, because the fitness vector holds only hit-rate. A pure latency change like v003 can only lose. Seven cycles have now flagged this.

## Open questions

- Does aging help if it is paired with single counting? Removing the double increment is what keeps post-reset zeros from turning into free admissions.
- Does conservative update plus a tie-break in the candidate's favour (`>=`, or `>=` only above a small floor) keep v002's hm_0 small gain and recover web_0? This would revisit the cycle 0004 tie-break verdict now that the sketch has changed.
- Can a per-op-time dimension be added to the evaluator, so that latency work becomes rewardable?

## Continuation

Keep cycle 0005 v003 as the base. No cycle 0007 variant beat it. Stop making one-knob sketch changes. Next cycle, try paired changes that keep the admission bar where it is: aging combined with single counting, and conservative update combined with a tie-break toward the candidate. Treat the sketch, the double count and the `>` rule as one coupled unit. Drop pure-latency hypotheses until per-op-time is measured, and keep independent row hashing.
