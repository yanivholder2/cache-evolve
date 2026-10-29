# Cycle 0007 Insight

## Cohort outcome

Two variants targeted latency and one targeted accuracy. v001 ported cycle 0006 v003's sketch prefetch and probe-before-increment reorder onto the merged line. v002 prefetched list neighbours ahead of `on_hit` promotion. v003 removed the duplicate sketch increment when a missed `get` is followed by `put` of the same key. All three are Winners by name only. None hit its main target. v001 barely eased get p99, possibly within noise. v002 regressed the tails. v003 gained hit-rate at the small size, lost more at the large size, and regressed latency across the board.

## Patterns

- Software prefetch has run its course on this line. With `increment_hashed` inlined, the sketch-block prefetch overlaps with the inlining gain instead of adding to it: get p50 stayed flat and put picked up a little extra work. Neighbour prefetch can't break the pointer chase, because `prev`/`next`/`region` live inside the hit node, which has to be loaded first. Both prefetch attempts leave hit-rate bit-identical, so hit-rate is no help in telling them apart. They live or die on latency, and both came up short.
- The get→put double count on a miss is not just an overcount. It also does two jobs the policy relies on: it speeds up the aging clock on miss-heavy traces, and it gives newcomers a small edge against the strict `>` admission test. Removing it helps the small size, where the window turns over fast, and hurts the large size, especially web_0. Small and large sizes again want opposite things, the same split seen in cycles 0004–0005.
- The idea that the large size needs better frequency evidence holds up, but v003 shows the evidence lever is tied to aging rate and tie-breaking. Any change to how often counts are added also changes how often the sketch resets.
- Unexplained latency swings on near-trivial diffs are back (v003). Per-op-time verdicts from single runs stay suspect.

## Open questions

- If v003's dedup is kept but skipped increments still count toward `additions`, or `SAMPLE_FACTOR` is lowered, does the small-size gain survive without the large-size loss?
- Would letting ties go to the candidate, or giving it a small explicit bonus, recover web_0 at the large size? That would separate the recency-bias effect from the aging effect.
- Is v003's latency regression caused by the `get` restructuring (`?` → `match` writing `self`), or is it noise?
- Can get p99 be cut by keeping link/region data next to the map value, so promotion loads start without first loading the node?

## Continuation

Drop software-prefetch variants, both sketch and neighbour, on the inlined merged line. Keep cycle 0006's v001 line as the base, since v001 here is effectively equal to it. Rework v003's dedup into a two-variable split: keep the aging clock unchanged (count skipped increments toward `additions`), then test the admission tie-break separately. Aim for the small-size gain without the large-size loss. If tail latency is pursued, change where link/region data lives in memory instead of adding hints, and confirm every latency verdict with interleaved runs.
