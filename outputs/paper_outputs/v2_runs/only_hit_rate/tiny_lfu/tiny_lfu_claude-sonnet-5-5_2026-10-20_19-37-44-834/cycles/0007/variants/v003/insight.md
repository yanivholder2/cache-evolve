# Variant v003 Insight

## Helped

- hit-rate: Nothing measurable. Sampling the two oldest probation entries and evicting the colder one should in theory keep hotter entries, but the result is indistinguishable from the parent.

## Hurt

- hit-rate: Marginally lower than the parent, within noise. Probation order is mostly recency order, and entries that have been promoted and demoted already carry similar sketch estimates. The second-oldest entry is therefore rarely much colder than the LRU, so the swap seldom changes the decision.
- Per-op cost (unmeasured): each miss now does up to two extra sketch estimates, each of 4 hashes. The vector reports only hit-rate, so this cost never shows up.

## Takeaway

Widening the victim choice from the probation LRU to the colder of the two oldest entries is effectively a no-op for hit-rate on workload-a at size 2048. A sample of two is too small to approximate LFU inside probation, and the admission duel was not limited by victim choice. Gains need to come from elsewhere, such as segment sizing or sketch aging, because the sketch here never decays. A larger sample would be needed to test the sampled-LFU idea properly.
