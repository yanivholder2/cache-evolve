# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants attacked the same weakness of pure LRU — it evicts a
reused key as readily as a one-hit-wonder — each via a distinct mechanism
(two-segment SLRU, midpoint insertion, CLOCK reference bit). Every variant
lifted hit-rate on this reuse-heavy LIRS trace; the segmented SLRU design
(v001) landed in the Winners.

## Patterns

- The winning lever was eviction *ordering*, not extra residency: every
  variant kept the capacity bound fixed and gained purely by deciding
  which resident key dies first.
- A "second access earns protection" signal — whether encoded as a
  protected segment, a position above the insertion midpoint, or a
  reference bit — is the common thread, suggesting the trace rewards
  distinguishing one-hit-wonders from genuine reuse.
- Cheaper, single-list approximations (midpoint insertion, CLOCK) also
  helped, so the gain does not strictly require two-segment bookkeeping —
  but the explicit segmentation won this cohort.

## Open questions

- Did SLRU win on margin, robustness, or both versus the lighter-weight
  single-list variants? The mechanism that scored highest is not
  obviously the one with the best cost/benefit.
- The 80/20 protected split in v001 was assumed, not tuned; the
  sensitivity of hit-rate to that ratio is unexplored.
- Whether the three protection signals are additive (e.g. segments +
  reference bit) or redundant is untested.

## Continuation

Pursue the "protect on second access" family — it is clearly the right
axis for this workload. Next cycle should tune within the winner (probe
the protected/probationary ratio and the promotion threshold) rather than
re-deriving that segmentation beats LRU. Consider one variant that
combines a cheap signal (reference bit or midpoint) with segmentation to
test additivity, and drop any further pure-recency baselines.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle probed three independent tweaks to last cycle's SLRU winner —
widen the probationary segment (H01), add a ghost list so post-eviction
reuses skip probation (H02), and require two hits before promotion (H03).
All three hurt hit-rate; none beat the parent. v003 (two-hit promotion)
landed in the Winners as the least-regressive, but every probe refuted its
own hypothesis.

## Patterns

- The parent's tuning was already near-optimal for this trace, and it
  errs in one consistent direction: **promote early, protect generously.**
  Loosening either lever lost hits — shrinking protected residency (H01),
  delaying promotion to a second hit (H03), and over-trusting a single
  post-eviction reuse (H02) all backfired.
- Reuse on this trace arrives *fast*: the second touch lands within the
  narrow 20% probationary window, so extra probationary dwell time (H01)
  is wasted and a stricter two-hit gate (H03) just loses keys that churn
  out before their second access.
- The binding constraint is protected-tier residency, not the strength or
  timing of the promotion signal. Anything that steals protected capacity
  or admits marginal keys into protected (H02's straight-to-protected
  ghost promotion) demotes genuinely hot residents and costs hits.

## Open questions

- The ratio should move *opposite* to H01 — does a larger protected tier
  (e.g. 90/10) keep winning, or is 80/20 already at the peak?
- Ghost/reuse-distance memory was directionally right but mis-placed:
  would routing ghost hits back to *probationary* (not protected) recover
  the signal without crowding the hot set?
- Every lever tested so far is monotone-worse than the parent; is v001's
  configuration a genuine local optimum, or are these three axes simply
  the wrong ones?

## Continuation

Stop loosening the parent — first-hit promotion and an 80/20-or-larger
protected tier are confirmed correct; drop further two-hit-gate and
wider-probationary probes. Next cycle should push protected residency the
*other* way (probe 85/15, 90/10) and, if revisiting ghosts, demote the
promotion target to probationary rather than protected. Treat v001 as a
strong local optimum and look for orthogonal levers (eviction-victim
selection, scan resistance) rather than re-tuning the promotion signal.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle pursued the three orthogonal levers cycle 0002 left open:
push the protected ratio further (H01, 90/10), frequency-windowed
eviction-victim selection (H02), and ghost re-admission with *weak*
(protected-LRU) protection (H03). Only the ratio push landed a clear
gain — v001 (90/10) won. The two scan-resistance / reuse-distance levers
worked directionally but barely moved hit-rate above noise.

## Patterns

- Protected residency remains the dominant lever. Confirming cycle 0002's
  predicted direction, enlarging the protected tier to 90/10 still beats
  the parent — more room for twice-touched hot keys converts directly to
  hits, and a 205-slot probationary segment still drains one-hit-wonders.
- The cleverer eviction-order machinery (frequency window, ghost memory)
  is nearly inert on this trace because second-access promotion already
  captures almost all the reuse signal *before* eviction order matters.
  Both H02 and H03 only rescue a thin band of warm-but-unpromoted keys.
- Cycle 0002's ghost placement diagnosis was correct: routing ghost hits
  to the protected *LRU* end (v003) recovers signal without crowding the
  hot set, where the prior protected-MRU placement regressed — but the
  recoverable headroom here is genuinely small.

## Open questions

- Is 90/10 the peak, or does the protected tier keep paying past it
  (95/5)? The probationary segment must stay non-empty for the eviction
  path, so there is a hard ceiling somewhere.
- The two near-flat levers (frequency window, weak-protection ghost) were
  tested in isolation; would either contribute once the ratio is at its
  optimum, or are they redundant with a maximally-protected tier?
- With promotion and ratio both near-tuned, has this trace's exploitable
  structure been largely consumed?

## Continuation

Keep pushing protected residency — probe past 90/10 (e.g. 95/5) to find
where the probationary floor breaks the eviction path; that is the live
lever. Treat frequency-windowed eviction and ghost re-admission as
confirmed-marginal on this trace and drop them as standalone probes. If
anything, test whether either adds value layered on the 90/10 winner
rather than on the original 80/20 parent. The cohort increasingly looks
near its ceiling on recency/promotion axes; consider whether a
fundamentally different workload signal remains untapped.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle ran the three threads cycle 0003 left open against the 90/10
SLRU winner: push the protected ratio to 95/5 (H01), layer a
weak-protection ghost on the incumbent (H02), and add a W-TinyLFU
admission window for scan resistance (H03). Only the ratio push won —
v001 (95/5) landed in the Winners, but by the thinnest margin yet. The
ghost was barely perceptible and the admission window actively
regressed.

## Patterns

- The protected-residency lever still pays at 95/5, but its return is
  now nearly spent: shrinking probationary to ~100 slots produced a far
  smaller gain than each prior enlargement. The binding constraint has
  shifted from "appetite for protected tenure" to the probationary floor
  needed to keep draining one-hit-wonders.
- Reuse-distance memory (the ghost) is directionally right but
  structurally redundant against a maximally-protected tier: with 90% of
  capacity already protected, genuinely hot keys are retained anyway, so
  the warm-but-evicted band left for the ghost to rescue is vanishingly
  thin — and it costs a ghost structure plus an O(n) scan per insert.
- A non-recency admission signal is not just inert here but harmful: the
  ~32-slot window is far smaller than this trace's reuse distances, so it
  evicts genuinely reused keys before their second touch. Scan resistance
  assumes one-hit-wonders dominate; on this LIRS trace they do not.

## Open questions

- 95/5 still pays but barely; is there any room left above it before the
  probationary segment can no longer host the eviction path, or is this
  effectively the peak?
- Both off-axis levers (ghost, admission window) have now been tested in
  isolation *and* layered on the incumbent and failed both times — is
  there any orthogonal signal in this trace left to exploit, or is the
  recency/promotion axis fully consumed?

## Continuation

Treat the protected-ratio lever as essentially exhausted: 95/5 is the
last marginal win and probably at or adjacent to the ceiling — do not
spend a future cycle on 97/3-style probes expecting real gains. Ghost
re-admission and front admission windows are now confirmed dead on this
trace (redundant or harmful); drop both permanently. The cohort has
consumed nearly all recency/promotion structure; the next cycle should
either accept the incumbent as a converged optimum or look for a
genuinely different workload signal rather than re-tuning SLRU geometry.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

After three cycles of exhausting SLRU geometry, this cycle finally
reached off the recency/promotion axis: an LFU protected-tier victim
selector (H01), a from-scratch canonical LIRS rebuild (H02), and a
front-vs-back demotion-placement tweak (H03). The LFU selector regressed
and the placement tweak was inert; v003 (demotion placement) landed in
the Winners as the neutral, least-regressive change. Notably, the LIRS
rebuild (v002) reported a hit-rate lift on its home-turf trace yet did
**not** win — the standout thread of the cycle.

## Patterns

- The recency/promotion axis is confirmed fully consumed. Two more
  in-skeleton levers (frequency at the demotion point, demotion
  placement within probationary) joined the pile of inert-or-harmful
  SLRU tweaks: at a near-full 95% protected tier, freq values are tied so
  LFU just mistracks recency and demotes recently-hot keys, and the
  binding constraint remains the probationary floor's *size*, not the
  order within it.
- Frequency is not the untapped axis cycle 0004 speculated about. Applied
  at the retention point it underperforms plain LRU here, mirroring the
  earlier admission-window failure — non-recency signals keep losing on
  this trace inside the SLRU frame.
- The one genuinely different mechanism — reorganising eviction priority
  around measured inter-reference recency (LIRS), rather than bolting
  re-admission onto SLRU — is the first off-axis probe to report a real
  gain. This is the "different workload signal" prior cycles kept asking
  for, and the trace family is its home turf.

## Open questions

- Why did v002's reported LIRS gain not land in the Winners? Was the lift
  within noise, or did the rebuild trip a contract/invariant check that
  disqualified it despite higher hit-rate? This must be resolved before
  trusting it.
- If the LIRS gain is real, does it dominate the 95/5 SLRU incumbent
  outright, or only on this specific LIRS-derived trace?

## Continuation

Stop tuning SLRU geometry and stop testing non-recency signals *inside*
the SLRU frame — both are exhausted. The live thread is the canonical
LIRS rebuild (v002): next cycle should re-run it cleanly, confirm whether
its reported hit-rate lift is real and beats the 95/5 incumbent, and
verify it preserves the observable contract. If it holds, make LIRS the
new parent and tune its LIR/HIR split; if it was noise or disqualified,
treat the cohort as converged on the 95/5 SLRU optimum. Do not revisit
LFU victim selection or demotion placement.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle followed up cycle 0005's two live threads: tune the LIRS
rebuild by transplanting SLRU lessons onto it (H01 widen the resident-HIR
window, H02 route re-admitted ghost hits to the weak tier), and close the
last SLRU geometry question by pushing the protected ratio to 97/3 (H03).
All three regressed. v003 (the 97/3 probe) won only as the
least-regressive change, and it did so by confirming a negative.

## Patterns

- The SLRU geometry axis is now formally closed at 95/5. Pushing to 97/3
  shrank the probationary reuse window below the floor needed to catch
  second touches, and the marginal protected slots were worth less than
  the drain capacity they consumed — the binding constraint is the
  probationary floor's *size*, exactly as prior cycles diagnosed.
- SLRU tuning lessons do not transplant onto LIRS. Both LIRS levers
  failed for structural reasons: the HIR window and LIR budget are
  zero-sum, so widening the reuse-detection queue directly evicts the
  low-IRR blocks LIRS was protecting; and a non-resident-HIR reuse is
  itself the canonical LIR-qualifying signal, so demoting it to the weak
  tier (the SLRU ghost-placement win) withholds protection LIRS correctly
  grants. LIRS's own S-membership and stack pruning already do the
  filtering SLRU bolted on after the fact.
- The cross-policy transfer hypothesis is disconfirmed: lessons learned
  inside one skeleton are not portable as parameter edits to another with
  a different invariant structure.

## Open questions

- Cycle 0005's central question — is v002's reported LIRS lift real and
  does it beat the 95/5 SLRU incumbent? — remains unresolved. This cycle
  only tried to *tune* LIRS (and failed both tunings); it never cleanly
  re-ran the untuned LIRS parent against the incumbent to confirm or
  refute the lift.
- With both LIRS tuning levers regressing from the parent config, is the
  LIRS parent at its own local optimum, or is there an untested lever
  (LIR/HIR warm-up, non-resident bound) that actually moves it?

## Continuation

Treat the SLRU line as fully converged at 95/5 — do not probe geometry
again. The one unfinished piece of business is the LIRS verification
cycle 0005 called for and this cycle skipped: next cycle should cleanly
re-run the untuned LIRS parent, confirm whether its hit-rate lift is real
and beats the 95/5 SLRU incumbent on its home-turf trace, and verify the
observable contract. Do not attempt further LIRS tuning by transplanting
SLRU parameters — that transfer is disconfirmed; tune LIRS only against
its own measured behavior. If the untuned LIRS lift does not survive a
clean re-run, declare the cohort converged on the 95/5 SLRU optimum.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle attacked three distinct, previously-untested levers against
the converged 95/5 SLRU incumbent: a third recency-of-Nth-access "hot"
tier inside protected (H01), an adaptive ghost-fed boundary that floats
the protected/probationary split at runtime (H02), and a widened LIRS
non-resident horizon (H03). All three regressed. v002 (adaptive
boundary) landed in the Winners only as the least-regressive change,
winning by confirming a negative.

## Patterns

- The 95/5 split is not just the best *static* point — it resists being
  improved *adaptively*. Letting the boundary float on reuse-distance
  feedback drifted to a worse partition than the constant it replaced,
  because the two nudge signals fire asymmetrically and the up-nudges
  erode the very probationary floor that all prior cycles identified as
  the binding constraint. The cache cannot discover a better split from
  reuse feedback alone here.
- Stratifying protected by reuse *depth* (LRU-K-like, not frequency)
  also fails: a depth-split protected segment starves its cold tier and
  churns out the twice-seen working set before it earns a third hit. A
  single flat protected LRU retains the reuse-heavy set better — reuse
  depth is not the binding signal on this trace.
- LIRS still has no demonstrated headroom. Widening the non-resident
  horizon surfaces exactly the long-distance, high-IRR second touches
  that should stay unprotected, and the unconditional non-resident-to-LIR
  promotion turns the extra memory into LIR pollution. The harm is in
  admission policy, not horizon length, so the capacity-bound horizon was
  already at or past optimum.

## Open questions

- Every off-axis lever now tried — depth stratification, adaptive
  geometry, LIRS horizon — has regressed. Is there any remaining
  structural axis on this trace, or is the incumbent a true converged
  optimum?
- The long-standing question from cycles 0005-0006 — does an *untuned*
  LIRS parent actually beat the 95/5 SLRU incumbent on a clean re-run —
  remains unresolved; this cycle only tuned LIRS (and failed) rather than
  verifying the parent.

## Continuation

Treat SLRU geometry as definitively closed: static is exhausted at 95/5
and adaptive resizing is now also disconfirmed — do not probe boundary
tuning again. Depth-stratified protected tiers and widened LIRS
non-resident memory are both dead; drop them. The single piece of
unfinished business is still the clean untuned-LIRS-vs-incumbent
verification that cycles 0005-0006 called for and no cycle has executed.
The next cycle should either run exactly that verification or, if it
again declines, declare the cohort converged on the 95/5 SLRU optimum —
three consecutive cycles of off-axis probes have now all regressed.


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
