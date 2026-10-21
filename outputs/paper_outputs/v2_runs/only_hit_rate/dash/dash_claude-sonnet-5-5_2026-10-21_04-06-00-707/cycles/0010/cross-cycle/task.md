# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants tweaked promotion logic in `DashSegment::get`/`promotion_get` against baseline: insertion position (v001), promotion eligibility into probing (v002), and probing→target back-promotion (v003). Only v001, the one-argument change to MRU insertion, won.

## Patterns

- Only hit-rate was measured. Metadata-size and per-op-time are unevaluated for every variant, so no cost claims exist yet.
- The winning lever changed where a promoted item lands, not how many items move. Variants that added movement (v002 more promotions, v003 bidirectional swaps) gained little or lost.
- Extra movement tends to cause demotion cascades toward stash and eviction. This offsets or exceeds the benefit of relocating items.
- The recency-skew rationale (don't make just-hit items the next victim) was supported by v001. Spare probing capacity was not a binding constraint (v002, marginal gain).

## Open questions

- Does MRU placement generalize beyond workload-a, and does it cost per-op time?
- Would combining v001 with v002 add anything, or is v002's marginal gain noise?
- Do other insertion positions (demotion path, stash→target vs. probing retries separately) matter?

## Continuation

Build on v001: explore insertion-position variants on other paths (demotion, cascade to stash) and test across more workloads, ideally with per-op-time and metadata in the fitness vector. Drop bidirectional probing→target swaps (v003) and don't expect more from eligibility relaxation alone (v002).


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three placement/eligibility tweaks were tried: MRU placement on the insert path (v001, parent cycle-1 v001), stash→probing promotion eligibility stacked on the MRU winner (v002, same parent), and cold-end demotion of displaced victims from baseline (v003). Hit-rate was unchanged from the parent in all three. v001 and v002 are listed as Winners, but only by matching the incumbent, not by improving on it.

## Patterns

- Every variant reproduced its parent's hit-rate exactly. That points to no-ops on workload-a at this cache size, or to overlays that never changed the executed path. It does not show the levers are harmful.
- The insert-path and eligibility levers add nothing beyond the hit-promotion position change. This supports the view that cycle 1's marginal eligibility gain was noise or already subsumed by MRU promotion.
- The cascade-eviction theory behind cold-end demotion got no support. Demotion position did not change which useful entries were lost.
- Only hit-rate was measured again. Metadata-size and per-op-time are still unevaluated, so there are no cost claims.
- Authors could not read the overlay sources, so the "overlay may not have taken effect" explanation could not be ruled out.

## Open questions

- Do these overlays actually alter the executed code path, or are they dead edits?
- Is workload-a or the cache size insensitive to placement on insert and demotion paths? Other traces or sizes might separate them.
- Does any variant differ on per-op-time or metadata, even with equal hit-rate?

## Continuation

The placement-position family (insert, demotion, eligibility) looks exhausted on workload-a. Drop further variants of it unless they are checked against a different workload or cache size. Next cycle should use a different lever, such as eviction victim selection, bucket and probing structure, or admission, branching from cycle/0001/v001. It should also add per-op-time and metadata-size to the fitness vector so ties can be broken.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three new levers were tried: victim selection by a furthest-from-MRU scan (v001), a second-sighting admission filter on the insert path (v002), and a wider probe neighbourhood to adjacent buckets before the stash (v003). Only the admission filter (v002) won, improving hit-rate on workload-a. Victim selection left hit-rate unchanged, and the wider probe lowered it slightly.

## Patterns

- The win came from deciding which keys get admitted. Cycles 1 and 2 changed where keys are placed, and that did not help. This points to one-hit-wonders as the main source of waste on this trace.
- Levers that delay or relocate eviction did not help. The wider probe let colder entries stay resident and crowd out hotter ones, which cost a little hit-rate.
- v001 reproduced its parent's hit-rate exactly, as the cycle 2 variants did. The edited victim-selection code may not be on the executed path, so this result is not clean evidence against eviction policy.
- Only hit-rate was reported again. The costs of v002's filter state and miss-path hashing, and of v003's extra probes, remain unmeasured.
- Authors still could not read the overlay sources, so every mechanism claim is inferred.

## Open questions

- Does the admission gain hold on other workloads and cache sizes? Is it worth the per-segment filter memory and the extra work on each miss?
- Is the filter's "full enough to need eviction" threshold tuned well, and would a different admission signal do better?
- Was the victim-selection edit live? Does a victim-selection change that provably runs behave differently?

## Continuation

Branch from v002 and keep pursuing admission: tune the threshold and filter design, and try frequency-based or doorkeeper-style variants. Drop wider-probe neighbourhoods (v003) and further placement tweaks. Put per-op-time and metadata-size in the fitness vector before the next cycle, so v002's gain can be weighed against its cost. Re-test victim selection only after confirming the edit is on a live code path.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants refined the cycle 3 second-sighting admission filter: a 2-bit saturating counter with aging (v001), a victim-conditional filter that admits first sightings when the cold-end victim was never hit (v002), and a small periodically-reset doorkeeper bit filter on the eviction path (v003). None improved hit-rate on workload-a. v002 is listed as Winner, but its own insight reports a hit-rate drop below the parent. It looks like the least-bad variant, not a real gain.

## Patterns

- Every refinement of the parent's unconditional one-bit filter lost hit-rate. The parent's design looks like a local optimum, and extra machinery added noise.
- Filter accuracy mattered more than filter sophistication. Smaller or faster-resetting state (halved slots, 256-bit filter wiped at half fill) raised collisions and forgot once-seen keys, so reusable keys were rejected again.
- A single "victim never hit" bit is a poor value signal. Un-hit entries are often still warm, and exempting them from the filter let one-hit-wonders back in.
- Only hit-rate was reported again. The metadata and per-op-time costs of the extra filter state and miss-path work remain unmeasured.
- Authors could not read the overlay sources, so the mechanisms are inferred.

## Open questions

- Is the parent's admission gain robust on other workloads and cache sizes, or specific to workload-a?
- Would a counter or doorkeeper pay off if the threshold or the eviction decision actually used the extra frequency information, with filter size kept constant?
- Why was a variant that lost hit-rate selected as a Winner? Is the winner rule relative to the cohort, or is the fitness vector missing dimensions?

## Continuation

Treat the cycle 3 v002 one-bit unconditional filter as the incumbent and stop tweaking its admission rule, victim exemption and repacked counters included. Next, explore a different lever: bucket and probing structure, or victim selection confirmed to be on a live code path. Alternatively, test the incumbent on other workloads and sizes. Add per-op-time and metadata-size to the fitness vector before any further filter work.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three variants were tried: a 4x larger, never-reset second-sighting filter on the cycle 3 incumbent (v001), a two-tier insert position that puts twice-seen keys at MRU and others next to MRU (v002), and a port of the one-bit filter onto the cycle 4 v002 variant (v003). v001 is the Winner, but only by a noise-level hit-rate gain. v002 was flat. v003 came out marginally below its parent.

## Patterns

- Every change landed within noise of its parent. Filter capacity, filter aging and insert position are all saturated on workload-a.
- Once the second-sighting filter screens out one-hit-wonders, the keys that get through are mostly worth keeping. Ranking them within a bucket adds nothing, which matches the cycle 2 finding that placement does not matter.
- The filter's gain did not stack on a different parent (v003). That parent already blocks some low-value inserts, so the two admission mechanisms overlap and are not independent wins.
- v001 bought a negligible gain with 4x the filter memory, so it is probably a bad trade. Memory is still unmeasured.
- Only hit-rate was reported again, and the authors could not read overlay sources. Cost claims and mechanism claims are still inferred.

## Open questions

- Is the Winner label for v001 meaningful, or is it noise-level selection again, as in cycle 4?
- Does the admission family hold up on other workloads or cache sizes, where filter capacity or placement might matter more?
- Is there any untried lever, such as bucket and probing structure or a different admission signal, that moves hit-rate more than noise?

## Continuation

Treat the cycle 3 v002 filter as the incumbent. Admission tuning (capacity, aging, counters, victim exemption) and insert-position tweaks are now exhausted on workload-a, so drop them. Don't stack admission filters on parents that already screen low-value inserts. Next cycle should try a structurally different lever (bucket and probing structure, or victim selection confirmed live). Alternatively it should validate the incumbent on other workloads and cache sizes. Add metadata-size and per-op-time to the fitness vector first, so noise-level winners like v001 can be rejected on cost.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three structurally different levers were tried: a reference-bit victim choice that evicts the first unreferenced slot (v001), power-of-two-choices bucket placement (v002), and a count-min frequency admission gate (v003). Only v001 won, with a clear hit-rate gain on workload-a. This is the first real movement since the cycle 3 admission filter. v002 was flat to marginally lower, and v003 gained only marginally.

## Patterns

- The headroom was on the eviction side. Cycles 1-5 worked on placement and admission and plateaued. A real reuse signal (hit since insertion) used for victim choice broke the plateau.
- Bucket-structure changes (v002) again did not matter, which matches the earlier finding that placement is saturated.
- A frequency-comparison admission gate (v003) added little, so admission is also near saturation. It overlaps with the incumbent's way of screening one-hit-wonders.
- v001 needs no extra metadata in principle, since the flag lives in spare bits. v003's sketch and v002's second probe add cost that is unmeasured.
- Only hit-rate was reported again, on a single trace, and the authors could not read the overlay sources. The size of v001's gain and all cost claims are inferred.

## Open questions

- Does v001's gain hold on other workloads and cache sizes, or is it specific to workload-a?
- Does a richer reuse signal do better than one bit, such as a multi-bit counter, CLOCK-style second chance or hand position? Does it stack with the admission filter?
- What is the CPU cost of the extra bucket and stash scan on eviction?
- Earlier cycles' victim-selection edits may not have been on the live path. Why did this one take effect?

## Continuation

Make v001 the new incumbent and branch from it. Pursue eviction using reuse information: multi-bit or aging reference state, CLOCK-style sweeps, and scan-resistance, while keeping the existing admission filter. Validate it on other workloads and cache sizes. Drop power-of-two placement (v002) and count-min or TinyLFU admission (v003), and do not add more admission machinery. Add per-op-time and metadata-size to the fitness vector so the eviction scan cost can be judged.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three refinements of the cycle 6 reuse-bit eviction incumbent were tried: a two-bit aged counter replacing the one-bit flag (v001), a changed victim scan order with second-chance clearing (v002), and letting keys bypass the admission filter when free slots exist (v003). Only v001 won, with a modest hit-rate gain on workload-a. v002 was identical to its parent, and v003 dropped slightly.

## Patterns

- Gains shrank sharply compared with the cycle 6 move. Most of the benefit came from using a reuse signal at all, and finer state (counters, aging) added only a little.
- Scan order and the all-referenced fallback (v002) had no effect. The eviction path is saturated in structure, not just in signal richness.
- Bypassing admission for free space (v003) hurt slightly. The admission filter is not redundant with reuse-bit eviction, because it also keeps cold keys out of empty slots.
- Aging counters on every scanned slot may cost CPU time. This is unmeasured, as is metadata size.
- Evidence still comes from one trace and one size, and authors could not read the overlay sources, so mechanisms and costs are inferred.

## Open questions

- Does the counter gain hold on other workloads and cache sizes, or is it noise-level on workload-a?
- Is the counter's eviction-scan cost worth a modest gain?
- With admission, placement and eviction all near saturation on workload-a, is any hit-rate headroom left on this trace?

## Continuation

Make v001 the incumbent but treat it as a small win. Drop scan-order and second-chance tweaks (v002) and admission-bypass ideas (v003); keep the admission filter unconditional. Further eviction-state refinement is now diminishing, so next cycle should validate on other workloads and cache sizes and add per-op-time and metadata-size to the fitness vector. Otherwise it should try a lever outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for different access patterns.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three refinements of the reuse-based eviction line were tried: a ghost table that restores a counter for recently evicted keys (v001), a non-mutating victim scan with periodic global counter halving (v002), and a second "protected" bit giving probation/protected tiers on the one-bit parent (v003). v001 won, but only by a very small hit-rate gain. v003 gained modestly over its parent, and v002 fell below its parent.

## Patterns

- Gains are small again. The reuse signal, the admission filter and the counter already capture most of what returning or reused keys offer on workload-a. Extra history (ghost table) and extra tiers (protected bit) add little.
- Replacing scan-time counter decrement with global timer-driven halving (v002) hurt. Local, scan-time aging is a useful idleness signal, so aging should stay tied to where the evictor looks.
- v001 and v003 both add state or scan work (ghost writes and lookups, multi-pass scan) whose cost is unmeasured. v001's near-noise gain is probably a poor trade for that.
- Evidence is still one trace and one size, with only hit-rate reported and no overlay sources readable, so mechanisms and costs are inferred.

## Open questions

- Is v001's Winner label meaningful, or noise-level selection as in cycles 4, 5 and 7?
- Does the SLRU-style two-tier idea (v003) stack with the counter incumbent, or does it overlap with it?
- Is any hit-rate headroom left on workload-a, or do other workloads and cache sizes need to be the test?

## Continuation

Treat v001 as a marginal incumbent at best. Drop global-timer aging (v002) and further history or ghost-state ideas. Eviction-state refinement (counters, tiers, scan order, aging) is saturated on workload-a. Next cycle should validate the incumbent on other workloads and cache sizes, and add per-op-time and metadata-size to the fitness vector so small wins can be judged on cost. If it explores new levers, pick ones outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three small refinements were tried: tighter ghost-table admission that skips cold victims and caps the restored counter (v001), dropping an incoming key when its whole bucket is hot (v002), and a compact one-bit reuse flag with a second-sighting bit-array admission filter (v003). v002 won, but its hit-rate gain over the parent was within noise. v001 slipped slightly and v003 gave only a marginal gain over baseline LRU.

## Patterns

- This is the fourth cycle in a row with noise-level gains. The Winner label here is probably selection noise, not a real improvement.
- Ghost-history tuning (v001) went the wrong way. Remembering cold victims appears to help, so filtering ghost entries by heat removes useful signal.
- Bucket-level scan resistance (v002) did almost nothing. All-hot buckets are likely rare on workload-a, or the counter and the second-sighting filter already protect hot entries.
- v003 shows a compact design reaches only a marginal gain. Its stated goals, metadata size and per-op time, were never measured, so its value is untested.
- The same gap remains as before. Only hit-rate on one trace and one size is reported, per-op-time and metadata-size are still missing from the fitness vector, and the authors could not read the overlay sources.

## Open questions

- Is any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- Would the compact v003 design be worth adopting if its cost savings were measured?
- Do scan-heavy or other workloads show a benefit from bucket-level protection (v002) that this trace hides?

## Continuation

Treat v002 as a marginal incumbent at best, and drop further ghost-table tuning (v001) and bucket-level admission blocking. Stop refining admission, placement, eviction state and ghost history on workload-a. Next cycle should first add per-op-time and metadata-size to the fitness vector, and validate the incumbent and v003's compact design on other workloads and cache sizes. If it explores new levers, choose ones outside those areas, such as adaptive sizing or scan-resistance for different access patterns.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Three variants were tried: a wider 3-bit reuse counter with the all-hot drop rule (v001), removal of the all-hot-bucket drop rule with a slot-position tie-break (v002), and an adaptive second-sighting admission gate with a reuse flag (v003). v002 was labelled Winner, but no variant showed a clear hit-rate improvement. The Winner label is probably selection noise again.

## Patterns

- This is the fifth cycle in a row with noise-level outcomes. Every change either regressed slightly or sat at the noise floor.
- More counter resolution (v001) hurt. Slow-decaying, finer state over-protects stale entries, so recency-responsive aging matters more than separating frequent from occasional reuse.
- The drop rule seems to give a small amount of real protection, since removing it (v002) cost a little. Its worth is marginal, and the simpler path's cost savings were never measured.
- Adaptive admission (v003) added moving parts (window counters, mode switch, unaged bit filter) and still lost to the plain baseline. Its trigger signal is laggy and noisy.
- The measurement gap persists. Only hit-rate on one trace and one size is reported. Per-op-time and metadata-size are missing, so simplification and complexity trade-offs cannot be judged.

## Open questions

- Is any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- Does the simpler v002 path win on per-op-time once that is measured?
- Do other workloads and cache sizes show effects that this trace hides?

## Continuation

Stop tuning counter width, drop rules, and admission gating on workload-a. Treat the incumbent as unchanged, and treat v002 as a simplification candidate only if cost data supports it. Next cycle should first add per-op-time and metadata-size to the fitness vector and validate on other workloads and cache sizes. After that, explore levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns.


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
