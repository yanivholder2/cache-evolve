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
