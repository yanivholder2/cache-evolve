# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three hypotheses made a single targeted change to the baseline's `promotion_get` path. One changed the stash-entry position for demoted items (v001), one added balanced placement on stash promotion (v002), and one reordered lookups (v003). All three became Winners. v001 gave the only clear hit-rate gain. v003 gave the only per-op-time gain that the code change itself can explain. v002 was safe but barely changed anything.

## Patterns

- All three variants report large get and put latency drops, including on paths their diffs never touched. The likely cause is a shared shift in the benchmark run or environment, not the code. This cycle's per-op-time comparisons against baseline can't be trusted, except for v003's get-path gain, which has a clear mechanism.
- Hit-rate moves when the policy changes how items are ordered for eviction inside the stash (v001). It barely moves when placement changes only in a rare corner case (v002). The stash holds most of the capacity, so its eviction order is the main hit-rate lever.
- Lookup cost is dominated by the stash scan. v003 shows that most hits are served from target/probing buckets, so checking those first is a free win. Hit-rate and metadata-size stay the same.
- None of the three changes affected metadata-size, and none conflict. They touch independent parts of the path: stash insert position, stash-hit destination, and lookup order.

## Open questions

- Is the stash mostly a holding area for misses, or does it serve real re-reads? v001 suggests demoted items do get reused, but v003 says hits mostly come from normal buckets. How hits split between the two tiers is still unmeasured.
- Promotion placement is still untested as a lever. This includes `get` inserting at Back vs. Front, and target-hit promotions using spare capacity in a neighbouring bucket.
- Would balanced placement at initial `put` time trigger often enough to matter, unlike the stash-hit case in v002?
- The baseline timing needs re-measuring under the current environment before any future latency claim can be trusted.

## Continuation

Combine v001's Back-of-stash demotion with v003's target→probing→stash lookup order as the new parent. They are independent and both are wins. Next, test promotion-placement position and balanced placement on initial put/target-hit paths, plus other stash eviction-order refinements. Drop further stash-hit-path placement tweaks, since v002 showed they rarely trigger. Re-measure baseline latency in the same run as candidates, and only credit per-op-time changes that have a code-level mechanism.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
