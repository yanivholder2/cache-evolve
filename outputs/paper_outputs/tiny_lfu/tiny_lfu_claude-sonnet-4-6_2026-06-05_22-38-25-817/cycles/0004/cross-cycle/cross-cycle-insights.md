# Cross-Cycle Insights

## Recurring wins

- **Eviction-side changes consistently beat admission-side changes.** Every winning variant has restructured how residents are ordered or selected for eviction; interventions at the admission gate (window sizing, sketch gating cadence, put-path triggers) have produced no lasting gains.
- **SLRU segmentation is the durable structural foundation.** The 80/20 → 90/10 protected/probationary split has held as the baseline across all cycles; the extra protected headroom prevents meaningful regressions even as other changes vary.
- **Victim-side frequency filtering (k=4 scan) opened a fresh positive axis.** Scanning k probationary tail candidates and evicting the minimum-frequency one rescues warm survivors that strict LRU displaces; this is the most recent structural gain and is not yet combined with the promotion-gate lineage.
- **The `was_protected` bypass makes aggressive promotion gating safe.** Distinguishing overflow-demoted items from genuinely cold newcomers allows the threshold to be set high without penalizing hot items caught in capacity churn; the pair is consistently more effective than either alone.

## Recurring failures

- **Sketch counter aging is harmful at every cadence.** Both a short interval (cycle 0001) and a 10× longer interval (cycle 0002) degraded hit rate. The frequency distribution on this trace is stable across the replay window; accumulated counts are signal, not noise, and periodic halving destroys that signal.
- **Ghost-set re-admission hurts on Zipfian workloads.** Tested in cycle 0003 (combined) and confirmed harmful in cycle 0004 (isolated): ghost-ring entries are precisely the items that lost the TinyLFU admission contest — they are genuinely cold — and fast-tracking them into the protected segment displaces legitimately hot residents. Mechanism is closed.
- **Capacity-cannibalization via fixed admission windows is immediately punishing.** Reserving a permanent segment shrinks the main LRU before any grace-period benefit can materialize; the trace does not produce items that accumulate frequency fast enough inside the window to recoup the structural loss.
- **Put-path promotion symmetry is a dead end on this trace.** The workload is read-dominant; hot items accumulate sufficient frequency via get() to qualify for promotion long before any put() trigger fires.

## Open questions

- Does layering the k=4 victim scan onto the PROMO_THRESHOLD + `was_protected` lineage yield additive gains, or does the promotion gate reduce probationary churn enough that the scan rarely overrides the default eviction choice?
- Is unconditional promotion specifically enabling the victim-scan benefit, or does the scan deliver similar gains regardless of the promotion policy?
- Is there a k value (e.g., k=8) that extracts more from victim-side filtering, or does the marginal benefit of a wider scan window shrink quickly?
- With admission-side and most eviction-side levers now explored, is there a remaining structural change — adaptive segment resizing, frequency-ordered ordering within protected, or a decay variant with a per-key counter floor — that can move the needle?

## Per-policy notes

- **TinyLFU admission gate:** Handles one-hit-wonders effectively on its own; it is not the primary bottleneck. No admission-side change has survived as a winner; this layer should remain untouched unless a hypothesis targets its interaction with a new eviction structure.
- **SLRU protected segment:** Internal LRU ordering and demotion-candidate selection are largely exhausted as levers. The promotion-threshold gate with `was_protected` bypass is the best-found configuration. Demotion-side frequency scanning (cycle 0004 v001) did not pay off; promotion-side gating (cycle 0003 v003) did.
- **SLRU probationary segment:** Functions as a pass-through staging area; it is not a competitive contention zone at 10 % capacity. The k=4 victim-scan is effective here and is the current frontier. Ghost-set bypasses originating from this stage consistently hurt.
- **Sketch counters:** Should be left to accumulate without decay on this trace. Any aging mechanism — regardless of interval — erodes the stable frequency signal that TinyLFU and the victim-scan both depend on.
