# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cycle probed three distinct levers — SLRU split-ratio tightening, sketch-aging at a much longer cadence, and a frequency-gated promotion threshold — against the SLRU baseline inherited from cycle 0001. Only the 90/10 split variant won, and only by a negligible margin; the other two produced no improvement or a slight regression.

## Patterns

- **SLRU ratio tuning has reached diminishing returns.** Moving from 80/20 to 90/10 barely moved the needle, confirming that the hot set was already well-contained in the protected segment at 80 %; the extra headroom covers only marginal-frequency items. The shrinkage of the probationary buffer caused no visible harm, reinforcing that probationary is a pass-through staging area on this workload, not a competitive contention zone.
- **Sketch aging is consistently harmful regardless of cadence.** Even at 10× the already-conservative reset interval tested in cycle 0001, periodic halving of counters erodes frequency signal more than it cleans up stale counts. The frequency distribution on this trace is stable across the entire replay window; accumulated counts are signal, not noise.
- **The promotion threshold landed in a behavioral dead zone.** At a threshold of 2, too few borderline items were affected — either they are too rare to matter or they accumulate frequency fast enough to qualify for promotion shortly after anyway. The concept is sound but the threshold value was ineffective at this workload's access rate.
- **All three axes hit a wall together.** None of the three hypotheses produced a meaningful gain, suggesting the SLRU-on-TinyLFU configuration is near a local optimum for workload-a at this cache size, and parameter-level tuning within the current structure is largely exhausted.

## Open questions

- Is there a promotion threshold high enough (e.g., 5–10) to actually reduce protected-segment churn, or does this trace simply not produce enough borderline-frequency contention for the gate to matter?
- What structural changes — beyond SLRU segmentation and sketch aging — could shift hit rate on a stable Zipfian workload? Candidates include ghost/victim caches, adaptive admission biases, or size-aware eviction.
- Is the admission frequency gate itself the tightest bottleneck, or is eviction ordering (within protected) the remaining lever?

## Continuation

SLRU split-ratio tuning and sketch aging are both exhausted as levers for this workload and should be deprioritized in future hypotheses. The next cycle should leave the 90/10 split in place (as the marginal winner) and pivot to structural rather than parametric changes: consider ghost-cache admission feedback (tracking recently evicted keys to bias re-admission), or probing whether the in-segment LRU ordering within protected could be replaced by a frequency-ordered variant. The promotion threshold concept is worth one more test at a substantially higher value, but should not consume a dedicated variant slot unless paired with a complementary change.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
