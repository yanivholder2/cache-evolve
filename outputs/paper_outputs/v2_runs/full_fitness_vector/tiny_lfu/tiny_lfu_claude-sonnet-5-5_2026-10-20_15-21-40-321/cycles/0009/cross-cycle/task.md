# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes to the baseline were tried: sketch aging (v001), a small admission window (v002), and a sketch rewrite with a single hash, u8 counters and conservative update (v003). Only v003 won. It improved speed and sketch footprint with a small hit-rate gain.

## Patterns

- The two changes that added mechanism to the admission path (aging, window) gained nothing on hit rate and cost per-op time and struct metadata. The change that made the existing sketch cheaper and more accurate was the only one that paid off.
- Hit-rate hypotheses premised on "stale frequencies" or "newcomers need a trial period" did not hold on workload-a. The baseline's never-decaying counters and immediate strict-greater admission looked well suited to this trace.
- The baseline's per-op cost is dominated by repeated hashing in the sketch. Removing it gave the clearest gain, especially in the put tail.
- Added fields or bookkeeping on the hot path (halving counters, segment lookup on get) raised latency without a matching benefit.
- The metadata-size summary tracks Node size, not sketch heap, so v003's real 4x sketch saving was invisible in the headline number.

## Open questions

- Do the u8 saturation cap and conservative update still hold up on traces with larger frequency gaps or drift?
- Would aging help with a much longer period, or on a trace with real frequency drift? Would a larger window help?
- Which of v003's parts (single hash, u8 counters, conservative update) account for the gains?

## Continuation

Build on v003 as the new parent and push further on per-op cost and footprint (cheaper hashing, tighter sketch layout, less work on the hit path). Drop the naive aggressive aging and the ~1% window from v001 and v002. Retry them only in a materially different form, such as a much larger window, conservative aging or a doorkeeper, and only with a rationale tied to a specific trace property. Prefer hypotheses whose gains show in the headline dimensions.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants were built on the cycle-0001 sketch: a single-mix hash with bit-sliced indexes and a cached hash (v001), a long-period counter halving (v002), and a 4-bit nibble-packed single-hash sketch (v003). All three were kept as Winners. Their gains were uneven: v001 was a clean speed win, v003 traded hit-rate for speed and footprint, and v002 gained little and lost hit-rate.

## Patterns

- Hashing and index derivation remain the main source of per-op cost. Both v001 and v003 cut it, and the put tail improved the most.
- Shrinking sketch precision (4-bit counters, one row, tight sizing) buys speed and footprint, but hit-rate falls noticeably. Changes that left counters, conservative update and admission untouched kept hit-rate essentially flat.
- Aging failed again, even with a period 10x longer. workload-a has stable frequencies, so decay discards useful history. Two cycles of evidence now point the same way.
- Every variant that added state to the cache struct grew it slightly. The headline metadata number tracks Node size, so these changes and the sketch heap savings do not show up there.
- v001's insight was written without access to the source, so its attributions (hash caching versus the mixing change) are inferred, not confirmed.

## Open questions

- How much hit-rate does v003's nibble layout recover with a second hash or a larger table, and at what speed cost?
- Is v003's loss mainly from the 4-bit cap, the single row, or the tight sizing?
- Does v001's speedup combine with v003's compactness, or does the cached-hash state offset the footprint gain?
- Is there any trace property that would justify decay? Without one, aging has no case.

## Continuation

Treat v001 as the preferred parent for speed: it has no hit-rate cost. Next, explore the v003 compact layout combined with v001's hashing, adding the minimum extra accuracy (a second hash or a larger table) needed to recover hit-rate. Drop periodic halving and any other aging variant unless there is a drifting trace to justify it. Target admission quality or sketch accuracy for hit-rate gains, and prefer hypotheses that move the headline dimensions.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three single-axis changes were tried: u32 list indices in place of `Option<usize>` links (v001), a two-candidate victim sample (v002), and a tie-admitting `>=` comparison (v003). v001 and v002 were kept as Winners. v003 was not. All three cost per-op time, and none gave a real hit-rate gain.

## Patterns

- The only change that moved a headline dimension was v001's layout shrink. It cut Node size with hit-rate unchanged, but it gave back per-op latency. The working set is small, so the locality benefit did not appear. Extra index conversion and sentinel checks seem to cost more than the saved cache lines.
- Both admission-quality changes (wider victim sample, looser tie rule) failed to lift hit-rate. The wider sample was marginal and probably noise. Admitting ties lowered hit-rate. Strict greater-than admission with the LRU head as victim again looks well suited to workload-a.
- Frequencies on this trace are low and often tied. Ties are common, so the tie rule matters, and loosening it removes protection for resident entries.
- Latency rose even where the hot path was untouched (v002's get, v003's one-operator edit). Some of the per-op regressions are probably measurement noise or code-layout effects, so small time deltas are unreliable evidence.
- Two of the insights were written without a source diff, so some of their attributions are inferred.
- A cheap victim-selection or comparison tweak is now ruled out as a hit-rate lever, adding to the earlier failures of aging and a window.

## Open questions

- Is v001's size gain worth its latency cost? Does the cost come from NIL checks and widening, or is it noise?
- Can the per-op regression be recovered, for example with unchecked indexing or a layout that avoids repeated conversion?
- Is there any signal that separates tied frequencies without adding hot-path work?
- How much of the observed latency variance is noise, and does it need repeat runs?

## Continuation

Keep v001 as the compact-layout parent, but aim next at winning back its per-op time (fewer conversions, tighter hot-path code) and combine it with cycle 0002's v001 hashing speedup and the compact sketch. Drop victim sampling and tie-admitting comparisons. Hit-rate gains need a genuinely new signal (a doorkeeper or sketch accuracy), not tweaks to the admission comparison or victim choice. Use a larger or repeated measurement before trusting small latency deltas.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three changes were tried on the compact-layout parent: a cheap map hasher with an MRU short-circuit on hit (v001), a one-bit doorkeeper in front of the sketch (v002), and a 4x wider sketch (v003). All three were kept as Winners. Only v001 was a real improvement: it cut per-op time and struct size with hit-rate unchanged. v002 and v003 moved hit-rate little or not at all.

## Patterns

- The map hasher, not link layout or the sketch, was the largest remaining latency cost. Swapping it recovered the per-op time lost to cycle 0003's compact layout and kept that layout's size gain.
- Both sketch-accuracy changes failed to yield a meaningful hit-rate signal. The doorkeeper slightly lowered hit-rate and widened the latency tail. Widening the sketch gave only a marginal gain at a large hidden heap cost. Sketch accuracy now looks saturated on workload-a.
- Adding state or hot-path work (a doorkeeper bitset) again raised tail latency and grew the struct. This repeats the pattern from cycles 0001 and 0002.
- v003's per-op improvement cannot be explained by its code change and is most likely noise. This is the second cycle where small latency deltas looked unreliable.
- Sketch heap changes stay invisible in the headline metadata number.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Does v001's gain hold up under repeat measurement, and which part accounts for it: the hasher or the MRU skip?
- Does v001 combine with the compact nibble sketch and cycle 0002's hashing speedup without losing hit-rate?
- With sketch accuracy, victim choice, tie rules, aging and windows all exhausted, is any hit-rate lever left on workload-a?

## Continuation

Use v001 as the parent and keep stacking per-op cost and footprint reductions on it (the compact sketch and cheap hashing from earlier cycles). Drop the doorkeeper and wider-sketch directions, since sketch accuracy is saturated and the wider sketch only trades heap for a marginal gain. Hit-rate ideas need a different mechanism entirely. Repeat latency measurements before crediting any small delta.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three footprint and speed changes were tried: reverting the 4x wider sketch on the cycle-0004 parent (v001), dropping a cached field from Node on the other cycle-0004 parent (v002), and a single-hash shift-derived sketch increment with a saturation early-exit, rooted at baseline (v003). All three were kept as Winners, but only v003 gave a plausible real gain (cheaper put and get medians, hit-rate level). v001 and v002 gave no real improvement.

## Patterns

- Sketch width is a weak lever in both directions. Narrowing saves hidden heap and costs a hair of hit-rate. The predicted locality speedup did not appear, and latency got slightly worse.
- v001's slowdown, and v002's get "gain" from an untouched hot path, are best read as noise. Together they cast doubt on the parent's earlier per-op improvement. This is the third cycle where small latency deltas looked unreliable.
- Removing a cached field from Node did not shrink it, because padding or alignment absorbed the saving. Moving the hash recompute onto the eviction path then taxed put tail latency. Node size has little room left to move this way.
- Cheaper sketch increment work (one hash, shift-derived indices, saturation early-exit) is the only speed idea this cycle that cleanly left hit-rate alone. It confirms the cycle-0002 finding that sketch cost is a main per-op term, now on an independent baseline lineage.
- Hit-rate stayed flat across the cohort. No variant touched a mechanism that could change it.
- Two insights were written without a source diff, so some attributions are inferred. v003 cannot separate the early-exit from the cheaper hashing.

## Open questions

- Does v003's increment speedup stack on the cycle-0004 v001 parent (cheap map hasher, MRU skip, compact links), or is it only visible against the baseline?
- Which part of v003 matters: the saturation early-exit or the single-hash derivation?
- Does any footprint lever remain once Node alignment is the floor, short of changing counter or key types?
- Is there any hit-rate lever left on workload-a?

## Continuation

Use v003's single-hash, shift-derived increment as the speed component and merge it onto the cycle-0004 v001 lineage; test the early-exit and the hash derivation separately. Drop sketch width tuning, and drop Node field removal unless it is paired with a layout change that crosses an alignment boundary. Repeat latency runs before crediting any small delta. Hit-rate gains still need a genuinely new mechanism.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three sketch-only changes were tried on existing incumbents: a single-hash shift-derived index (v001, on the cycle-0004 v003 parent), a saturation early-exit alone (v002, on the cycle-0004 v001 parent), and conservative-update increments (v003, on the cycle-0005 v002 parent). All three were kept as Winners, but none gave a real improvement. Hit-rate and metadata size stayed flat, and per-op time did not improve.

## Patterns

- The cycle-0005 single-hash speedup did not carry over to the stronger lineage. On this parent it made `get` slightly slower and may have cost a hair of hit-rate, since slicing indices from one hash correlates the rows. The earlier gain was therefore specific to the baseline lineage.
- The early-exit alone did nothing measurable. Counters rarely sit at the cap, so the early-exit rarely fires. Any speedup in the cycle-0005 variant that combined both changes probably came from the hash derivation, and that derivation has now failed to stack.
- Conservative update left hit-rate unchanged. Collision inflation is not what limits hit-rate on workload-a, which is the third sketch-accuracy result (doorkeeper, width, conservative update) to say the same thing.
- Per-op shifts across the cohort were small and in both directions. Most are best read as noise, as in earlier cycles.
- Sketch cost no longer looks like the main per-op term on the current incumbents. Sketch tweaks of every kind (accuracy, width, increment cost) are now exhausted.
- Two insights were written without a source diff, so some attributions are inferred.

## Open questions

- Where does `get` actually spend its time on the incumbent: map lookup, list relinking, or sketch estimate? No profile exists.
- Is any remaining gain in the non-sketch hot path (relinking, allocation, bounds checks), or are the incumbents at the floor for this design?
- Is there any hit-rate lever left on workload-a, given that sketch accuracy, victim choice, tie rules, aging, windows and doorkeeping have all failed?

## Continuation

Drop all sketch-internal changes (hash derivation, early-exit, conservative update, width, doorkeeper). Aim the next cycle at the non-sketch hot path of the cycle-0004 v003 incumbent (map access, list relinking, bounds checks), or at a structurally different admission mechanism such as a window or segmented layout, and have each hypothesis name which cost it expects to move. Repeat latency runs before crediting any small delta.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three non-sketch changes were tried on different parents: pre-sizing the map and slab with in-place slot reuse (v001), inlining and branch hints on the `get` fast path (v002), and a segmented probation/protected main LRU on the baseline (v003). All three were kept as Winners. Only v003 showed a plausible real change: a smaller Node and faster get and put, with a slight hit-rate loss. v001 and v002 were flat or worse.

## Patterns

- Allocation and growth are not a bottleneck. Pre-sizing and slot reuse changed nothing measurable, because the cache fills early and steady-state growth cost is negligible.
- Forcing inlining and branch hints on helpers shared by `get` and `put` did not help and may have slowed both. Call overhead, duplicate hashing and branch layout are not where `get` spends its time. The broad slowdown may also be noise.
- Segmenting the main LRU did not raise hit-rate. The sketch admission filter already supplies most of the scan resistance this trace needs, so main-region scan resistance is not the hit-rate limiter.
- v003's speed and size gain came from the compact Node, with segment membership packed into a spare bit, not from the segmentation itself. This again points to memory footprint and locality as the lever that pays.
- Hit-rate was flat or slightly down across the cohort, so the plateau on workload-a holds against a structural mechanism too.
- Two insights were written without a source diff, so some attributions are inferred. Small latency deltas remain unreliable without repeat runs.

## Open questions

- Is v003's speed gain driven by the compact Node alone? If so, can it be reproduced on the stronger incumbents without the segmentation and its hit-rate cost?
- Where does time actually go in `get` and `put`? There is still no profile.
- Is any hit-rate lever left on workload-a in window sizing or the admission decision itself?

## Continuation

Drop pre-sizing, `#[inline(always)]` and branch-hint tweaks, and drop segmenting the main LRU as a hit-rate idea. Pursue footprint and locality: carry the compact 24-byte Node layout onto the best incumbent without the segmentation, and test whether the speed gain survives repeat runs. For hit-rate, look at window sizing or the admission decision, and have each hypothesis name which cost it expects to move.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three changes were tried: halving the sketch table (v001), shrinking the window to a smaller share of capacity (v002), and a compact Node with u32 slab-index links and packed flags, without segmentation (v003). All three were kept as Winners, but none improved speed or hit-rate. Only v001 and v003 gave a real footprint reduction, and each paid for it with slower get and put.

## Patterns

- Footprint reductions did not turn into speed. Both size cuts, one in the sketch and one in the Node, left latency worse. The working set seems small enough that locality gains are absent. The index-conversion and flag-unpacking overhead is paid on every operation.
- v003 undercuts the cycle-0007 lesson. The compact Node alone gave no speed gain, so cycle 0007's speedup came from something else, or was noise. The "footprint and locality pay" thread is now weak.
- Window size is not a hit-rate lever. A smaller window lost a little hit-rate and slowed put, since more inserts go through admission and eviction. This closes the window-sizing question from cycle 0007.
- Hit-rate has stayed flat or slightly down again across the cohort. Sketch, window, segmentation and Node layout have all failed to move it on workload-a.
- v002 showed unexplained growth in Node and struct size that the change should not cause, so its overlay may carry an unintended edit. Its attributions are inferred.
- Latency regressions are consistent in direction across v001 and v003, which makes them somewhat more believable than earlier noise-level deltas. Winner status here reflects the selection rule, not real gains.

## Open questions

- Where does time go in get and put? There is still no profile, and every blind hot-path guess so far has been flat or worse.
- Is the admission decision itself (compare incoming frequency against victim frequency) the only remaining hit-rate lever, or is workload-a at its ceiling for this design?
- Are the incumbents at the floor for this architecture, so that further gains need a different data structure?

## Continuation

Drop sketch size, window size, and Node compaction as standalone levers. They are all explored. Do not build on v002 without auditing its diff. Next cycle should either obtain a profile-driven target (map lookup, relinking or sketch estimate) before proposing speed changes, or try a structurally different admission or data-structure idea, such as an adaptive window or a cheaper hash map. Each hypothesis should name the specific cost it expects to move and be checked with repeat runs.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three hypotheses were planned (adaptive window, relink-skip plus victim-estimate cache, flat open-addressing table with a slab and single hash). v001 was rejected: its implementer hit a backend "prompt too long" failure and produced nothing, so the adaptive window is still untested. Winners were v002 and v003. v003 gave a real, large latency gain. v002 was a regression that passed only on the selection rule.

## Patterns

- The only substantive win is a storage-layer change. Replacing the pointer-based map with a flat linear-probing table over a slab, hashing once per operation and reusing that hash for the sketch rows, cut get and put latency at median and tail. Hit-rate, Node size and policy stayed unchanged.
- This is the first blind hot-path guess in several cycles that paid off, and it contradicts the cycle-0008 worry that incumbents sit at the floor. The map and hashing path, not sketch, list or Node layout, was the dominant per-op cost on the baseline.
- v002 confirms that relinking and repeated victim estimates are not the cost. The epoch is bumped on nearly every access, so the victim cache almost never hits, and the bookkeeping slowed both paths and grew the cache struct. Hit-rate was unaffected.
- Hit-rate remains flat across every cohort, including a pure storage change. Metadata size is dominated by the sketch, so storage work cannot move it.
- v003's parent was the baseline, not the strongest hit-rate incumbent, so its gain is not yet shown to stack on the later lineages.

## Open questions

- Does the flat-table and single-hash gain carry onto the cycle-0004 v003 or cycle-0007 lineages? Cycle 0005 showed that a baseline-only speedup can fail to transfer.
- Is the v003 gain mostly from the flat table or from the single hash feeding the sketch? Earlier single-hash derivation failed to stack on strong parents.
- Does the adaptive window do anything for hit-rate on workload-a? It was never evaluated.

## Continuation

Port v003's slab and flat-table storage onto the best hit-rate incumbent, and separate the table's effect from the single-hash reuse, with repeat runs. Re-propose the adaptive hill-climbing window (H01) once, since it was never tested. Drop the victim-estimate cache and near-head relink skipping, and keep hypotheses aimed at the map and hashing path rather than the sketch or list.


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
