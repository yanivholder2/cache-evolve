## Recurring wins

- Activating latent capacity in the existing data model (LFU counter already carried at zero cost) produced the first win with minimal structural change and no regressions.
- Composing two complementary mechanisms — LFU eviction and a second-chance stash — yielded mutually reinforcing gains: LFU pre-filters the stash toward items with real reuse history, so stash promotions are productive far more often than a naive second-chance mechanism alone.
- Wins consistently come from operating at structural boundaries (the eviction decision, the stash-to-primary promotion transition) rather than tuning internals of a mechanism already working well.
- Changes that are structurally minimal and additive (rather than replacement-based) have a clean win-or-neutral profile with no regressions.

## Recurring failures

- Eviction-scoring refinements (frequency counter caps, frequency-times-recency product scores) repeatedly measure as neutral on workload-a because the hot set is stable — the failure modes these target (counter inflation, insertion-pressure edge cases) do not occur on this trace.
- Enlarging stash capacity regressed hit rate: a larger cap lets cold items accumulate long enough to earn spurious promotions that displace medium-frequency primary residents.
- Applying LFU selection inside the stash regressed hit rate: every item in the stash is already the lowest-frequency item from primary, so the frequency gradient within the stash is too compressed for LFU to discriminate; FIFO age-ordering is better matched to the stash's role.
- Counter-derived heuristics at the promotion boundary (frequency-proportional credit, intake filtering by counter value) regressed hit rate: on Zipfian workloads, an item's LFU counter at eviction time reflects burst-displacement timing far more than genuine future reuse probability.
- Filtering stash intake to exclude zero-counter items regressed hit rate: zero-counter items at eviction are recently inserted items displaced before the workload could access them again, not cold waste.

## Open questions

- The `metadata-size` dimension has been absent from fitness vectors for four consecutive cycles. The stash adds per-bucket `Vec` fields whose memory overhead has never been quantified; variants targeting memory efficiency remain unverifiable until this is resolved.
- Would counter-cap, product-score, and other neutral-on-workload-a hypotheses show meaningful gains on a more adversarial trace with hot-set drift or bursty insertion patterns?
- With the stash-and-promotion subsystem now well-calibrated, what is the next structural axis outside this subsystem that has gone unexplored?
- Is there a workload-b or synthetic adversarial trace that would expose failure modes invisible on workload-a's stable Zipfian hot set?

## Per-policy notes

- **ClassicLRU (baseline):** Starting point; had slack in both frequency and recency dimensions simultaneously, making it susceptible to cold-burst displacement of hot items.
- **LFU eviction only (cycle 1 winner):** Structurally minimal; no regressions; reduced cold-burst displacement immediately.
- **LFU eviction + second-chance stash (cycle 2 winner, established baseline):** FIFO stash at bucket-quarter capacity; mutually reinforcing composition; this pairing is now the stable foundation across all subsequent cycles.
- **LFU + stash + promotion credit +2 (cycle 3 winner, current best):** Small but clean win; +2 clears the tie that caused promoted items to be immediately re-evicted on the next cold burst; operates at the stash-to-primary interface rather than inside either mechanism.
- **Promotion credit +3 and frequency-proportional scaling (cycle 4 regressions):** Confirm +2 is the local optimum; the promotion-credit axis is exhausted.
- **Stash intake filter (cycle 4 regression):** Confirms all recent evictions should enter the stash unconditionally; counter-value-based filtering at intake is counterproductive.
