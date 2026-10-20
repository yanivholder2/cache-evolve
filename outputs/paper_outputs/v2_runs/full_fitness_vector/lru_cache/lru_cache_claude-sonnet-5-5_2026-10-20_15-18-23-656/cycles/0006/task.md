# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three variants refined the segmented LRU: u32 links on the fast segmented cache (v001), segmentation built onto the u32-link plain LRU with the segment bit packed into `next` (v002), and lazy demotion moved from `get` to `put` (v003). All three were Winners, but each paid for it in per-op-time. Nothing improved speed, and the only gains were metadata size (v001) and hit-rate (v002).

## Patterns

- Every variant slowed per-op-time, including tails. This contrasts with cycle 0002, where u32 links and the hasher were pure wins on the plain LRU. Once the policy is segmented, link-width and bit-packing changes no longer bring speed.
- Narrowing links shrank the cache (v001), but the added widening and NIL checks, plus the hash fold, cost latency. Locality did not pay off at this workload size. The saving is size only, and it is not confirmed that u32 links on a segmented policy are a speed win.
- v002 again showed that the cost of segmentation is the promotion/demotion relinking on the hot path. Packing the flag keeps per-node size flat but cannot remove that work. The container grows from the extra head/tail pair, and a per-node metric hides this.
- v003 showed that deferring demotion only moves the relink work. It landed on `put`, the more latency-sensitive path, and `get` did not improve. Hit-rate was preserved.
- The remaining time cost of segmentation is the promotion/demotion pointer surgery itself, and reshuffling it between operations does not fix that.
- Variant authors could not read the diffs. Attribution among the hash fold, index conversion and flag masking is inferred.

## Open questions

- Which piece of v001 caused the slowdown: the hash fold or the u32 conversions? The fold was bundled with the layout change.
- Can promotion be made cheaper without eliminating it, for example by promoting only when a node is far from the head, or by promoting on a sampled or deferred basis?
- Can the segment be stored implicitly, such as by list membership, so that no masking is needed on link access?
- Does segmentation's hit-rate gain justify its latency cost on workloads other than the scan-heavy trace?

## Continuation

Stop shifting demotion work between `get` and `put`, and stop bundling the hash fold with layout changes. Next, run an isolated test of the fold against the u32 conversion cost. Then attack promotion cost directly: skip the relink for nodes already near the head, or batch or amortize promotion. Treat the cycle-0002 merged candidate (segmented, u32 links, folded Fx, move-based eviction) as the baseline, and judge new variants by whether they recover speed without losing hit-rate.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three attacks on segmentation's remaining cost were tried: a head-adjacent relink skip for protected hits (v001), a CLOCK-style second-chance reference bit replacing relink-on-get (v002), and removal of the hash avalanche fold (v003). All three were Winners. The real gains were v003's put speed and smaller struct, and v002's modest hit-rate gain with cheaper get. v001 was neutral on hit-rate and slower.

## Patterns

- The hash fold was a measurable cost on the segmented variant, mostly on put. A single multiply plus rotate gave enough mixing with no hit-rate change. This answers the cycle-0003 question: the fold, not the u32 conversion alone, was a major source of the slowdown.
- Cheaper promotion by skipping relinks only for nodes at or near the head did not pay off. Few hits land there, so the added branch cost more than it saved. Any promotion skip must cover a large share of hits, or add nothing to the hot path.
- Removing relink from get (v002) moved the work to eviction. Get got slightly faster, put tail got worse, and node size was unchanged. This is the same work-shifting pattern seen with lazy demotion in cycle 0003, though here the get hot path really did shrink.
- A single reference bit gave only a small scan-resistance gain, so it is real but limited.
- v003's struct-size drop was not targeted, which suggests the change touched more than the hash arithmetic. Attribution is inferred, since authors could not read diffs.

## Open questions

- Does the second-chance bit compose with the fold-free hasher, and do their gains add up?
- Can the eviction walk in v002 be bounded or amortized to cut the put p99 without losing the hit-rate gain?
- Why did put slow in v001, which was not its target? Possible causes are a shared touch path, code layout, or noise.
- Does the v003 struct shrink hold, and what caused it?

## Continuation

Combine v003's fold-free hasher with v002's reference-bit get path as the next baseline. Then bound the eviction scan, for example by capping the number of referenced nodes passed per put, to recover put p99. Drop head-adjacent relink-skip ideas and further hasher tweaks. Check hit-rate on workloads other than the scan-heavy trace.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three small refinements of the cycle-0004 baseline were tried: the fold-free hasher combined with the reference-bit get path (v001), a capped second-chance eviction walk (v002), and a larger protected share of the segmented cache (v003). All three were Winners, but the gains were small. Only v002 clearly changed the trade-off, and it paid in hit-rate.

## Patterns

- Combining the fold-free hasher with the reference-bit path (v001) preserved eviction order and left hit-rate and size unchanged. Speed moved only marginally and may be noise, so the two cycle-0004 gains compose weakly. The hash is now a small share of per-op cost.
- Capping the eviction walk (v002) cut get and put latency, including put p99, which was the cycle-0004 target. It did so by coarsening recency, which lowered hit-rate. Latency and hit-rate now trade off directly through the cap.
- Raising the protected share (v003) slightly lowered hit-rate on the scan-heavy trace. A smaller protected segment and a larger probationary one look at least as good, so the split is a weak lever.
- v003 edited only a construction-time constant, yet its latency shifted. This points to layout or measurement noise at the nanosecond scale, so small per-op-time deltas across the cohort should not be credited to the changes.
- Variant authors could not read the code, so all attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Can the put p99 gain be kept with a larger or adaptive cap, or with a cheaper rule for choosing which referenced nodes to spare, so hit-rate recovers?
- What is the noise floor of per-op-time, and which recent "speed wins" fall below it?
- Does a smaller protected share improve hit-rate beyond the baseline split?
- Does hit-rate behave the same on workloads other than the scan-heavy trace? That check has been pending for several cycles.

## Continuation

Treat v001's combination (fold-free hasher plus reference bit) as the baseline, and explore the cap in v002 as a latency/hit-rate dial, looking for a setting or an adaptive rule that keeps put p99 low without losing hit-rate. Probe a smaller protected share rather than a larger one. Drop further hasher and constant-tuning ideas that sit within noise. Measure a noise floor and evaluate on non-scan workloads before accepting small speed claims.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
