# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants targeted the protected segment from different angles — frequency-guided demotion selection, ghost-set fast-track re-admission, and promotion gating with a demotion-history bypass. Only the promotion-gate variant (v003) was selected; all three produced neutral-to-positive results, reinforcing that the configuration is near a local optimum where incremental gains are small.

## Patterns

- **Promotion-side filtering beat demotion-side selection.** Replacing the scan-based demotion picker (v001) with a threshold gate on *promotion* (v003) was the only change that cleared the winner bar. At a 90/10 split the protected segment's LRU tail is already high-frequency, so searching for a colder demotion candidate rarely overrides the default LRU choice; blocking borderline items from entering protected in the first place yields more.
- **The `was_protected` bypass is the essential partner to aggressive gating.** Raising the promotion threshold without distinguishing overflow-demoted items from frequency-weak newcomers would penalize genuinely hot items caught in capacity churn. The bypass makes the threshold safe to set high.
- **Ghost-set re-admission is real but buffer-size-constrained.** The ghost-ring mechanism produced a positive gain with no regressions, but at a buffer capped near capacity/8 the fast-track path fires infrequently because evicted items cycle out of the ring before they are re-requested. The mechanism and the SLRU changes were introduced together, leaving their individual contributions unresolved.
- **The cohort produced no regressions.** With three variants all landing flat-to-positive, the cache is operating well within a safe region, but the absence of a decisive winner signals diminishing returns from structural tuning of the protected segment alone.

## Open questions

- Does the ghost-set mechanism from v002 add further gain when layered on top of v003's promotion gating, or do the two interact and neutralize each other?
- What ghost-buffer size (e.g., capacity/4 or capacity/2) is needed for the fast-track path to fire frequently enough to matter on this trace?
- Are admission-side changes — adjusting the TinyLFU frequency gate itself — the remaining meaningful lever, given that protected-segment internals appear nearly exhausted?

## Continuation

Take v003's 90/10 SLRU with promotion threshold and `was_protected` bypass as the new baseline. The next cycle's highest-value experiment is isolating the ghost-set mechanism on top of v003 (not combined with a fresh SLRU variant) to determine whether ghost-set re-admission contributes independently. If that confirms a real gain, a follow-on probe on ghost-buffer size is warranted. Protected-segment demotion scanning should be deprioritized; admission-side or decay-side hypotheses are the better structural frontier if ghost-set isolation also disappoints.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cycle tested ghost-set isolation on the prior lineage, put/get promotion symmetry, and a clean-slate SLRU rebuild with frequency-sampled victim selection. Only the baseline-rooted variant — SLRU 90/10 with a k=4 LRU-tail frequency scan and unconditional promotion — produced a gain and was selected winner.

## Patterns

- **Victim-side frequency filtering opened a new positive axis.** Where cycles 0001–0003 exhausted the promotion and admission sides, scanning the k=4 probationary LRU candidates and evicting the minimum-frequency one rescued warm survivors that strict LRU would have displaced. The gain was modest, consistent with the scan overriding the default choice only at the tail of the k=4 window.
- **Ghost-set is definitively harmful on Zipfian.** Fast-tracking ghost-buffer hits into protected degrades hit rate because ghost-ring entries are precisely the items that already lost the TinyLFU admission contest — they are genuinely cold, and elevating them displaces legitimately hot residents. This mechanism has now regressed twice and the failure mode is well-understood.
- **The put-path promotion gap is a workload fiction on this trace.** Symmetric promotion in put() produced zero movement because the trace is read-dominant: hot items accumulate enough frequency via get() to qualify for promotion long before any put() could act as a trigger. The change is logically sound but structurally irrelevant here.
- **Winning from the baseline reset signals the lineage may have over-fit.** v003 (H03) was rooted at the baseline, not the accumulated cycle-0003 lineage (PROMO_THRESHOLD=8, `was_protected`, 90/10 split). That a baseline-rooted variant outcompeted two variants built on the prior winner suggests the PROMO_THRESHOLD gate and victim-scan may interact — either reinforcing or interfering — and the combined design space has not been explored.

## Open questions

- Does layering the k=4 victim scan onto the cycle-0003 lineage (with PROMO_THRESHOLD and `was_protected`) yield additive gains, or does the promotion gate reduce probationary-churn enough that the scan rarely changes the eviction choice?
- Is unconditional promotion (used in the winning variant) specifically enabling the victim-scan benefit, or is the scan effective regardless of promotion policy?
- With ghost-set ruled out, is there any remaining admission-side lever worth testing, or is the eviction path now the primary frontier?

## Continuation

The next cycle should build on the winning variant (SLRU 90/10, k=4 victim scan, unconditional promotion) and attempt to graft the PROMO_THRESHOLD gate from cycle-0003's lineage onto it — the two approaches have not been tested in combination and the interaction is unresolved. If that combination regresses, it confirms the gate and the scan are redundant on this workload and the simpler scan-only design is preferable. Ghost-set should not be revisited; it has a clear failure mechanism on Zipfian and is now closed. Put-path symmetry is also a dead end for this trace. Increasing k in the victim scan (e.g., k=8) is a low-cost probe if the combined variant disappoints.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants pushed against the edges of the cycle-0004 winner — combining promotion gating with the victim scan, widening the scan window, and enlarging the protected segment. Only the combined promotion-gate variant (v001) was selected, and then by a vanishingly small margin; the other two produced no measurable change.

## Patterns

- **Promotion gating and victim scanning are compatible but not synergistic.** Layering PROMO_THRESHOLD + `was_protected` onto the k=4 scan did not produce additive gains — on this Zipfian trace most probationary items either accumulate enough frequency to clear the threshold quickly, or carry the `was_protected` flag and bypass the gate anyway, so the gating collapses toward unconditional promotion in practice.
- **The victim-scan window is fully saturated at k=4.** Doubling to k=8 changed no eviction decisions, confirming that the coldest reachable candidate already appears within the first few LRU-tail positions. The scan's bottleneck is not window width.
- **The SLRU split-ratio axis is conclusively closed.** The gain trend from 80/20 → 90/10 does not continue to 95/5; the hot working set fits comfortably within the existing protected budget and the additional headroom rescues nothing.
- **Three simultaneous null/marginal results signal a local optimum.** The current design — 90/10 SLRU, k=4 victim scan, PROMO_THRESHOLD gate, `was_protected` bypass, TinyLFU admission — has exhausted all parametric tuning axes that have been probed across five cycles.

## Open questions

- Is frequency-ordered eviction *within* the protected segment (replacing its internal LRU order) a meaningful axis that has not yet been tested?
- Could tuning the TinyLFU admission gate itself — rather than post-admission segment management — provide the next increment, given that the segment internals are now near-optimal?
- Does the marginal win from v001's promotion gate hold at different cache-size-to-working-set ratios, or is it purely an artifact of the tight capacity used in this benchmark?

## Continuation

Adopt v001 (90/10, k=4 victim scan, PROMO_THRESHOLD≥8, `was_protected` bypass) as the new baseline, but treat all five tuning knobs as fixed — none warrant further adjustment. The next cycle should pivot to a qualitatively different structural change: the most promising unexplored axis is frequency-aware ordering within the protected segment (evicting the cold tail of protected rather than the LRU tail). If that also disappoints, the admission gate and sketch configuration are the remaining structural frontier. All prior closed axes — ghost-set, put-path symmetry, scan-window widening, split-ratio changes — should not be revisited.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
