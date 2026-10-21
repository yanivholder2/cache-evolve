# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three hypotheses were tried against the baseline: compact u32 arena links (H01), a segmented LRU (H02), and a faster hasher with cheaper touch/eviction paths (H03). The Winners were the segmented LRU (v002), which gained hit-rate, and the Fx-hasher/touch variant (v003), which gained median latency and a smaller struct. The compact-links variant (v001) was rejected before evaluation because the simulator build failed on a Maven download rate limit (HTTP 429). That is an infrastructure failure, so H01 is untested.

## Patterns

- The two Winners improved different axes: v002 improved hit-rate through scan resistance, and v003 improved per-op-time and struct size with no hit-rate change. They are largely independent and could be combined.
- Both Winners paid for their gains in tail latency. Promotion/demotion list surgery hurt put p99 in v002, and weak hashing hurt get p99 in v003. Median gains came with worse tails in this cohort.
- v002 also increased metadata (a larger node and extra list bookkeeping). v003 reduced it. Metadata-size is a real axis that policy changes tend to worsen.
- v003 only partly landed H03: the key clone on the eviction path in `put` remained, so some of the available put savings were left unused.

## Open questions

- Can the hash be made fast without the tail regression, for example with a better mixing or fold step?
- Can segmentation's metadata cost be recovered, for example by encoding the segment bit in existing fields or using compact links?
- Does the v002 hit-rate gain hold on traces other than the scan-heavy lirs workload-a?
- Does H01's u32 links idea deliver its locality and size benefit once it is evaluated?

## Continuation

Re-attempt H01 (u32 sentinel links), since v001 failed for infrastructure reasons and not on merit. Build on the Winners: combine segmentation with the fast-path touch and a better-mixed hasher, and finish removing the eviction-path key clone. Prioritize fixing the tail regressions (get p99 from hashing, put p99 from promotion/demotion). Do not spend more effort on proving that segmentation helps scan workloads; that is established.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three hypotheses landed as Winners, and none regressed hit-rate. H01 combined the segmented LRU with the fast hasher, the cheap touch path and removal of the eviction key clone. H02 packed the segment flag into a link field to shrink the segmented node. H03 re-ran the u32 sentinel-link idea, with a hash avalanche fold, on the baseline. H03 is the first test of u32 links, because it was never evaluated in cycle 0001.

## Patterns

- Layout and mechanics changes (u32 links, a lighter hasher, a head-touch early return, a key move) were pure wins that left eviction order alone. Hit-rate came only from the policy, and speed and size came only from mechanics. The axes stayed independent.
- The two changes that shrank metadata were the u32 links in H03 and the segment bit packing in H02. Only H02 paid a per-op-time cost, from mask and accessor work on every link access. H03's size gain came with a speed gain.
- The avalanche fold on the Fx hash appears to have avoided the get p99 regression seen with the weaker hash in cycle 0001. That tail fix has not been confirmed on a segmented variant.
- In H01, put gained most, in both median and tail, so the removed clone and the hasher likely account for most of it. The touch fast path helps only when hits land on the segment head.
- The variant authors could not read the code diffs. Attribution within each bundled change set is inferred, not measured.

## Open questions

- Do u32 links and the segment-bit packing compose? The u32 links change the spare-bit budget the packing relies on.
- Does the hash fold also fix the put p99 and get p99 tails once it is on the segmented policy?
- Which of H01's bundled pieces (hasher, touch fast path, clone removal) carries the speed gain?
- Does the segment-bit masking cost vanish with u32 links, or can the segment be stored implicitly, for example by list membership?

## Continuation

Merge the three branches into one candidate: segmented LRU with u32 sentinel links, the folded Fx hasher, the head-touch early return and the move-based eviction. Pack the segment bit only if its masking cost can be kept off the put path. Next, target the remaining promotion/demotion tail on put, and check hit-rate on workloads other than the scan-heavy trace. Stop re-proving that mechanics changes preserve hit-rate, and stop re-proving that segmentation helps scans.


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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three small policy tweaks on the cycle-0005 v003 parent were tried: a larger protected share (v001), demoted nodes reinserted at the LRU end of probation (v002), and `put` on an existing key no longer promoting (v003). All three were Winners, but the changes were tiny. Only v001 gained anything real, a small hit-rate improvement.

## Patterns

- The protected-share trend keeps improving hit-rate across steps, but each step gains less. The split is a weak lever with diminishing returns.
- Demoting a protected node to the next-to-evict position hurt hit-rate slightly. Demoted nodes have shown reuse and are better treated as warm, not cold.
- Skipping promotion on `put` updates left hit-rate unchanged. The trace rarely overwrites resident keys, so hit-rate is decided by the `get` and miss-insert paths.
- Metadata size was flat across the cohort, as expected for edits that touch no layout.
- Per-op-time deltas appeared even where hot paths were untouched (a constant-only edit, an untouched `get`). These are noise, which confirms the cycle-0005 suspicion that small latency deltas are not attributable. Winner status here likely reflects noise tolerance, not improvement.
- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Where does the protected-share trend turn, if it does? Or is it saturating?
- Is there any hit-rate lever left in segment-boundary policy, or does the remaining gain lie elsewhere (admission, sizing, ghost history)?
- What is the actual per-op-time noise floor, and would a repeat-run measurement separate real deltas from noise?
- Does any of this hold on workloads other than the scan-heavy trace? This check has been pending for many cycles.

## Continuation

Drop `put`-update semantics, further demotion-position tweaks and more protected-share constant steps. They are exhausted or in the noise. Next, try a structurally different hit-rate lever, such as an admission filter or ghost-entry history for scan resistance, and use this cycle's v001 (larger protected share) as the baseline. Measure the noise floor and evaluate on non-scan workloads before crediting any speed claims.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two ghost-history variants and one micro-optimization were tried: ghost-hit keys sent straight to the protected segment (v001), cold keys inserted at the LRU end of probation unless the ghost table remembers them (v002), and an evicted-key clone removal in `put` (v003). All three were Winners, but only v002 produced a real gain, a hit-rate improvement paid for with slower put. v001 improved nothing, so its Winner status looks like noise tolerance, not merit.

## Patterns

- Ghost history works as a scan filter on insertion position, not as a promotion trigger. Demoting cold keys to the next-to-evict slot (v002) helped hit-rate. Promoting ghost hits into protected (v001) hurt it, because returning evictees are mostly not hot enough to displace protected entries.
- Both ghost designs cost the same on the miss and eviction path: an extra hash, a random table access and side memory. This showed up as worse put, especially at the tail. v002 left get unchanged. v001 slowed both.
- The ghost table lives outside `Node`, so the per-node size metric hides its memory cost. Metadata-size comparisons undercount this family of changes.
- v003 confirms again that key-copy micro-optimizations are invisible. For a plain integer key, the clone is free, and put cost is dominated by map and list work. Its small latency shifts are within noise.
- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Can the ghost cost be cut, for example by hashing only on eviction and reusing the hash already computed for the map, or by a smaller or sampled table, so put p99 recovers?
- Does a middle path work: ghost hits inserted at probation MRU, with no protected admission and no LRU-end insertion for cold keys?
- How much of v002's gain depends on the scan-heavy trace? Does it hurt or help on non-scan workloads?
- What is the real noise floor of per-op-time? This has been pending for several cycles.

## Continuation

Use v002 (ghost-gated LRU-end insertion) as the next baseline. Aim at cutting its put and memory overhead, such as a cheaper fingerprint, a smaller table, or a reused hash, and at checking hit-rate on non-scan workloads. Drop ghost-to-protected promotion (v001) and key-clone or other micro-optimizations that sit inside the noise.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three low-risk refinements of the cycle-0007 ghost-gated baseline were tried: a halved ghost table (v001), a larger protected share of the segmented cache (v002), and fewer redundant hashes in `put` (v003, ghost lookup gated on a full cache and the victim hashed once). All three were Winners. The gains were small. The ghost-cost work reduced put tail latency, and the constant tweak gave a tiny hit-rate gain.

## Patterns

- Ghost overhead can be trimmed without losing the scan-filter benefit. A smaller table (v001) and less hashing (v003) both cut put tail latency while hit-rate held. The cycle-0007 ghost design was oversized and redundant.
- Both ghost-cost wins reach only the put tail. The median put is dominated by hash plus map insert, and `get` is untouched. Further gains need structural changes, such as fewer map operations per put, not fewer hashes in a fixed sequence.
- The protected-share trend still gives hit-rate gains, now on top of ghost-gated insertion, but they are tiny. It remains a weak, low-risk knob with diminishing returns.
- Per-op-time deltas again appeared on paths the change cannot touch (get after a ghost-only edit, get p50 after a constant-only edit). This is noise, so small timing deltas stay unattributable.
- The metadata-size metric is blind to heap-allocated ghost memory, so the claimed memory savings are unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- How far can the ghost table shrink, or become sampled, before hit-rate degrades? The current evidence only shows that halving is tolerable.
- Can the number of map operations per put be cut, to move the median put and not just the tail?
- Does the ghost-gated design, with its larger protected share, hold up on non-scan workloads? This check has been pending for many cycles.
- What is the per-op-time noise floor? Repeat-run measurement is still missing.

## Continuation

Treat the v001 and v003 combination (smaller ghost table plus single-hash gated eviction path) as the baseline. Pursue structural put-path reductions, such as fewer map operations, and add a heap-inclusive memory measure so ghost cost is visible. Drop further protected-share constant steps and hash or clone micro-optimizations, since they are saturated or within noise. Evaluate on non-scan workloads and measure a noise floor before crediting small speed claims.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried: a second halving of the ghost table (v001), in-place reuse of the evicted node's slot in `put` (v002), and ghost-hit keys inserted at probation MRU or second position (v003). All three were Winners, but none showed a verified gain. v001 held hit-rate level, v002 improved nothing, and v003 lowered hit-rate. Winner status looks like noise tolerance.

## Patterns

- The ghost table still has headroom. A second halving did not cost hit-rate, but put tail latency did not improve either, so shrinking it no longer buys anything measurable.
- Cutting the allocate/free pair in `put` (v002) did not help. Map remove and insert dominate put cost, so node allocation is not the lever.
- A middle path for ghost hits (v003) hurt hit-rate. The ghost filter helps by keeping suspicious keys cheap to evict, not by rewarding remembered ones. Both ghost-hit promotion routes (protected last cycle, probation MRU now) have failed.
- `get` timing moved in all three variants, mostly at the tail, although `get` was never edited. Run-to-run or layout noise swamps small per-op-time deltas, as in every cycle since 0005.
- The metadata-size metric cannot see heap-allocated ghost memory, so the memory saving from ghost shrinking remains unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- What is the per-op-time noise floor? Without a repeat-run measure, no latency claim can be credited or blamed.
- Does the ghost-gated design generalize to non-scan workloads? This has been pending for many cycles.
- Can map operations per put be reduced structurally, for example with a single combined lookup/insert (entry-style API) or a cheaper map?
- Is the remaining hit-rate gain in admission or sizing, since insertion-position and segment-share levers are exhausted?

## Continuation

Treat cycle-0008's ghost-gated baseline, possibly with the smaller ghost table, as the parent. Drop ghost-hit insertion position, ghost shrinking, node-slot reuse, and protected-share steps. Next cycle, build a noise-floor and heap-inclusive memory measurement, and evaluate on non-scan workloads. Then try a structurally different lever: fewer map operations per put, or an admission policy.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Three structural or admission-style probes were tried: fusing the map remove and insert in the evicting `put` (v001), a ghost-counter admission gate that protects probation victims (v002), and second-chance eviction with an adaptive walk cap (v003). v001 and v002 were Winners, but neither showed a verified gain. v001 left hit-rate and layout unchanged and v002 left hit-rate unchanged. Winner status again looks like noise tolerance. Only v003 improved hit-rate, slightly, and it was not a Winner because it cost latency and metadata.

## Patterns

- Every change that added ghost, counter or reference-bit bookkeeping to the put path made put slower and grew state. The admission and adaptive-cap variants (v002, v003) both did this, and neither bought a clear gain.
- The parent's cold-key insertion rule seems to already capture most of the useful admission signal. A second admission layer (v002) changed no eviction decisions on this trace.
- Fusing map calls (v001) did not speed up put. Together with last cycle's node-slot reuse, this suggests map-call count and allocation are not the bottleneck, or that a single-probe version costs about as much as remove plus insert.
- `get` timing moved in variants that did not meaningfully touch `get` (a large tail drop in v001). This is the same noise seen in every cycle. The noise floor is still unmeasured.
- Metadata growth leaked past the stated layout targets in two variants (struct growth in v002, node and struct growth in v003), despite the hypotheses planning for none.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- What is the per-op-time noise floor? Without it, no latency claim can be credited or blamed. This has been pending for many cycles.
- Does the ghost-gated design hold on non-scan workloads? The check is still pending.
- If map calls and allocation are not the put bottleneck, what is? Hashing, cache misses on list pointers and the eviction walk are untested.
- Is any further hit-rate left on this trace, or is the baseline near its ceiling?

## Continuation

Keep the cycle-0008 ghost-gated baseline as the parent. Drop ghost-counter admission gates, adaptive walk caps and second-chance bits, since they add put cost and state without a verified payoff. Also drop further map-call fusion and allocation tweaks. Next cycle, build the noise-floor (repeat-run) and heap-inclusive memory measures and evaluate on non-scan workloads. Profile where put time actually goes before choosing another lever.


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
