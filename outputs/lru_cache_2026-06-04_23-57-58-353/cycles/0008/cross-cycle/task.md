# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants tested reuse-aware eviction policies against a scan-heavy workload; the two-segment SLRU design (v001) was the sole Winner, producing the largest hit-rate gain by explicitly partitioning entries that have proven their usefulness from those that have only been seen once.

## Patterns

- Every variant improved hit rate over the baseline — none regressed — confirming that stock LRU's scan-pollution vulnerability is real and addressable by multiple independent mechanisms.
- The cohort converged on a single root cause: baseline LRU does not distinguish one-hit entries from reused ones, so cold-traffic bursts displace warm entries. All three strategies attacked exactly that axis, just at different points in the eviction pipeline.
- SLRU (v001) won decisively because it acts at insertion time, routing every new entry into a probationary buffer before it can touch the protected working set. CLOCK (v002) and the frequency-window scan (v003) act only at eviction time, which is a later and weaker signal.
- The frequency-window gain (v003) was explicitly bounded by the fixed 8-slot look-ahead; entries just outside the window remained vulnerable regardless of their access count, leaving a structural ceiling on improvement.

## Open questions

- Can CLOCK or frequency-window eviction be layered *inside* the SLRU protected segment to add a second line of defense once entries are promoted?
- Would a larger or adaptive frequency-window (beyond 8 candidates) close the gap with SLRU on this workload, or does the eviction-time limitation make it a dead end regardless of window size?
- The CLOCK variant eliminates linked-list pointer maintenance entirely — does that structural simplification yield measurable throughput benefits under high concurrency or very large capacities?

## Continuation

Next cycle should advance SLRU (v001) as the new baseline and probe the 80/20 protected-to-probationary split ratio — a tighter or looser split may further tune the balance between scan resilience and working-set capacity. It is also worth exploring whether layering frequency-based demotion on top of SLRU (evicting low-frequency entries from the protected segment back to probationary rather than strict LRU demotion) closes any remaining gap; the frequency-window idea is not exhausted, just constrained by its current eviction-only placement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort probed three directions off the cycle-1 SLRU winner: loosening the segment-size ratio, layering frequency-aware demotion inside protected, and adding a ghost admission set. Only the ghost-admission variant (v003) won, by combining two-segment SLRU with cross-residency warmth tracking.

## Patterns

- The 80/20 protected-to-probationary split has now been stress-tested from both sides: cycle-1 confirmed it beats plain LRU, and this cycle confirmed that widening probationary to 30% hurts — the hot working set is large enough that the smaller protected segment triggers more demotion churn than the extra scan-filtering headroom saves.
- Frequency-aware demotion inside protected produced no measurable effect when the scan window covers only a small fraction of the segment and the frequency counter resets to zero on every demotion. The signal evaporates before it can accumulate durable discrimination between high- and low-reuse entries.
- Ghost admission targets a different gap than in-cache frequency: keys evicted from probationary that re-arrive before the ghost window expires have effectively earned two accesses across residency periods and deserve direct entry into protected. On a LIRS-style trace where that pattern is common, this signal is worth capturing.
- v003 was built from the baseline rather than from the cycle-1 SLRU winner, so its measured gain bundles both SLRU segmentation and ghost admission; the individual contribution of the ghost set is not yet isolated.

## Open questions

- How much of v003's gain is attributable to ghost admission alone versus SLRU segmentation? Applying ghost admission to the cycle-1 SLRU baseline (without rebuilding the segment logic) would isolate the effect.
- Could frequency-aware demotion recover if the scan window were much larger and the counter were persistent across demotion cycles rather than reset? The current failure is almost certainly a design flaw rather than a fundamental limit of the idea.
- Is ghost capacity equal to probationary capacity the right size, or would a smaller ghost window (e.g., half of probationary) suffice while reducing bookkeeping overhead?

## Continuation

Next cycle should take v003 as the new baseline and add ghost admission on top of the established cycle-1 SLRU structure as a control variant, to cleanly measure the ghost set's marginal contribution. It is also worth retrying frequency-aware demotion with a larger scan window and non-resetting counters before abandoning the idea; the cycle-2 failure was a signal-design problem, not proof that frequency is uninformative in the protected segment. Further ratio tuning of the 80/20 split is likely a diminishing return and can be deprioritized.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort probed three orthogonal directions off the cycle-2 baselines: isolating ghost admission's marginal contribution, rescuing frequency-aware demotion with a larger scan window and persistent counters, and testing a second-chance reprieve bit in the protected segment. Only the repaired frequency-demotion variant (v002) won, delivering a meaningful hit-rate gain by scanning the LRU quarter of protected and halving rather than resetting counters on demotion.

## Patterns

- Ghost admission is now confirmed as minimally effective on this workload. When grafted cleanly onto the cycle-1 SLRU baseline, it produced negligible lift, resolving cycle-2's ambiguity: the earlier gain attributed to ghost admission was almost entirely SLRU segmentation doing the work. The ghost window (bounded to the probationary capacity) is too narrow to intercept the reuse gap typical of this trace.
- Cycle-2's frequency-demotion failure was entirely a design flaw, not a fundamental ceiling. Scaling the scan window to a fixed fraction of the protected segment — rather than a constant small count — creates genuine selection pressure. Counter halving on demotion is the necessary complement: it lets accumulated history survive segment transitions so newly promoted nodes remain distinguishable from long-tenured ones.
- The second-chance bit failed because it required freezing node position on protected hits, which severed the continuous recency signal that makes LRU ordering useful. A one-time reprieve cannot substitute for a move-to-tail on every access; the hot entries it was meant to protect end up stranded near the demotion head anyway.
- Continuous position updates on protected hits are load-bearing. Any mechanism that suppresses them — regardless of its reprieve logic — degrades the protected segment's ordering and should not be adopted unless the position-signal loss is compensated elsewhere.

## Open questions

- Can frequency-aware demotion (v002) coexist with ghost admission without noise? Given ghost admission's negligible effect on workload-a, the combination is likely neutral, but the interaction has not been tested directly.
- Is the LRU-quarter scan fraction robust across workload shapes, or does the right window size vary with the reuse distance distribution? A workload with longer reuse distances might need a wider scan.
- Could a second-chance approach be salvaged by setting the referenced bit while still performing the move-to-tail (preserving recency ordering)? The current failure was the position-freeze trade-off, not the reprieve concept itself.

## Continuation

Next cycle should take v002 as the new baseline: two-segment SLRU with ghost admission, frequency-aware demotion scanning the LRU quarter of protected, and counter halving on demotion. Ghost admission can be deprioritized — it has been tested in isolation and found to contribute little on this workload. A productive direction is to explore whether the scan fraction (currently one quarter) should be adaptive or tuned, and whether combining frequency-aware demotion with a significantly larger ghost window (e.g., equal to total capacity rather than probationary capacity) can make cross-residency warmth visible enough to matter.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants extended the cycle-3 frequency-demotion baseline in different directions — scan-fraction widening, ghost-window expansion, and frequency-aware eviction in probationary. Only the probationary extension (v003) won, by ensuring that the freq counter preserved across demotion actively shields recently-warm entries at both eviction sites.

## Patterns

- The frequency counter and counter-halving-on-demotion design have been the load-bearing mechanism since cycle 3, but it was applied asymmetrically: protected-segment demotion respected frequency, while probationary eviction discarded it via blind LRU. v003 closed that gap — the signal survived the segment boundary but was being thrown away at the next eviction decision.
- Scan-fraction tuning above 1/4 in the protected segment is now confirmed to offer negligible returns on this workload. Going from 1/4 to 1/3 moved hit rate only marginally, indicating the frequency variation near the LRU end of protected is mostly captured by the narrower window already.
- Ghost-window expansion to full capacity regressed hit rate: reuse events at distances between probationary capacity and total capacity on this trace are not reliable locality signals, and fast-pathing those keys into protected pollutes it. The effective ceiling for the ghost window sits well below total capacity, and the ghost-window expansion thread is now largely exhausted.

## Open questions

- Is the probationary LRU-half scan fraction robust, or should it be adaptive? A workload with longer reuse distances or a more skewed access distribution might need a different fraction at the probationary eviction site.
- With frequency awareness now applied at both eviction sites, is there any remaining structural gap, or have the major policy improvements been captured? A direct comparison against a wider workload suite would clarify how general these gains are.
- Could an intermediate ghost window (e.g., 50% of total capacity, between probationary capacity and the failed full expansion) recover some cross-residency signal without the admission-quality problem seen at full capacity?

## Continuation

Next cycle should take v003 as the new baseline: two-segment SLRU, counter-halving demotion, frequency-aware eviction scanning the LRU quarter of protected and the LRU half of probationary. Ghost-window expansion can be deprioritized — the full-ceiling test failed and intermediate values are unlikely to move the needle meaningfully on this trace. The most productive direction is probing the probationary scan fraction (is 1/2 near optimal, or does a smaller fraction suffice?), and testing whether either or both scan fractions should vary proportionally with observed reuse-distance patterns rather than being fixed.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed the cycle-4 frequency-aware baseline from different angles — scan-fraction narrowing, counter inflation control, and admission gating — and none produced a hit-rate gain. The winner (v002) advanced on neutrality: the frequency cap introduced no regression while the other two variants hurt performance, making it the best performer in a cohort that collectively confirmed the current configuration is near a local optimum on this workload.

## Patterns

- The probationary half-scan is now confirmed as genuinely necessary, not merely conservative. Frequency variation among probationary entries — cold newcomers mixed with demoted entries carrying halved counters spanning a wider range — is spread throughout the list, not compressed near the LRU head. The quarter-scan that suffices for protected (where all entries have at least one access and frequency variation near the LRU end is compressed) does not transfer to probationary.
- Counter inflation is not an active problem on this workload. The halving-on-demotion mechanism already prevents runaway accumulation in practice: entries must remain in protected without ever being demoted for counters to inflate far above the cap, and that residency pattern does not occur at this cache size on workload-a. The MAX_FREQ=8 cap is structurally sound as a guardrail but targets a pathology the trace does not exhibit.
- The frequency-aware eviction scan introduced in cycle 4 already provides sufficient cold-traffic filtering in probationary. A two-hit promotion gate is redundant protection — the entries that survive in probationary are generally worth promoting immediately, and the extra exposure window costs more hot-key residency than the one-hit-wonder reduction saves.
- Two consecutive cohorts (cycles 4 and 5) have produced wins that are structural closures rather than fresh gains: cycle 4 closed the asymmetry between eviction sites; cycle 5 confirmed that three further refinements of the resulting design produce no improvement. This is a convergence signal for the current policy on this workload.

## Open questions

- Is the policy now at a local optimum for workload-a, or are there unexplored structural directions (e.g., adaptive scan fractions tied to observed hit-rate feedback, segment-ratio tuning, or load-sensitive demotion) that could break out of the current plateau?
- Would the frequency cap (v002's change) matter on a workload with longer-lived hotspots where continuous protected residency without demotion is more common?
- Is widening the evaluation to a second workload the most informative next step — to distinguish genuine convergence from workload-specific saturation?

## Continuation

Next cycle should carry v002 forward as the new baseline (the MAX_FREQ=8 cap is a harmless guardrail worth keeping). The probationary half-scan and immediate single-hit promotion are both settled — neither narrowing the scan nor adding a promotion gate is productive on this workload. With three scan-fraction and admission variants now exhausted, the highest-value direction is either a genuinely new structural axis (adaptive policy parameters, segment-ratio exploration beyond the previously tested range, or a different eviction philosophy) or explicit multi-workload evaluation to determine whether the policy has converged broadly or only on workload-a.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort revisited three previously-identified threads — a referenced-bit tie-breaker in the demotion scan, an intermediate ghost window at half of total capacity, and logical-clock frequency decay at victim-selection time — and none produced a hit-rate gain. The winner (v001) advanced on neutrality for the second consecutive cycle, confirming the policy is at a workload-a local optimum.

## Patterns

- The referenced-bit tie-breaker (v001) is structurally sound and avoids the cycle-3 position-freeze failure, but it targets a collision pattern — equal-frequency candidates where one is recently referenced — that this trace does not produce at the narrow LRU-quarter scan window. The mechanism is inert rather than harmful.
- The ghost window is now exhaustively mapped across the meaningful range: probationary capacity is too narrow to matter, half-capacity regresses, full capacity (cycle 4) regressed harder. Reuse events at distances between probationary capacity and half-total-capacity are cold traffic on this workload, not deferred-warm traffic, and the ghost mechanism cannot be rescued by window sizing alone.
- Logical-clock decay at a fixed half-life (v003) is acutely workload-sensitive. The value of the existing frequency signal comes from distinguishing demoted-from-protected entries from cold newcomers; a half-life shorter than the typical inter-access gap on this trace collapses that distinction and eliminates the scan's ability to protect warm entries — the same problem it was meant to sharpen. Any fixed decay constant is unlikely to generalize without being tuned to the workload's reuse-distance distribution.
- Two consecutive cohorts (cycles 5 and 6) have produced winners that advance on neutrality rather than gain. This is a convergence signal: the main structural gaps in the policy have been closed, and the remaining unexplored axes either require workload-specific tuning or are genuinely unproductive on workload-a.

## Open questions

- Is there any fixed-rate decay constant (e.g., half-life at 2048 or 4096 ops) that preserves the frequency distinction on workload-a, or does the inter-access distribution make any single constant wrong?
- With ghost admission, scan fractions, promotion gating, frequency caps, referenced bits, and decay all exhausted, are there structural eviction philosophies not yet tested — adaptive segment sizing, ARC-style balance feedback — that could break out of the current plateau?
- Is the policy broadly strong, or specifically tuned to workload-a? Multi-workload evaluation is now the clearest way to answer this.

## Continuation

Next cycle should treat the current policy (frequency-aware demotion and eviction, counter halving on demotion, MAX_FREQ cap, optional referenced-bit tie-breaker) as a stable baseline and pivot away from parameter tuning on workload-a. Ghost window and fixed-rate decay are both exhausted as productive directions. The highest-value move is either introducing a second workload to test transferability of the gains, or exploring a genuinely different eviction philosophy — such as adaptive segment-ratio balancing — that does not share the local optimum of the current SLRU design.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Both tested variants abandoned frequency counting entirely — one replacing the full policy with SIEVE's visited-bit second-chance mechanism, the other switching to inter-reference recency (IRR) timestamps within the existing SLRU structure. The IRR-based variant (v003) won with a substantial hit-rate lift, breaking out of the three-cycle neutral-winner plateau.

## Patterns

- Switching the eviction signal from accumulated frequency counts to recency-of-recency (IRR) is the sharpest gain yet recorded. The LIRS-generated trace rewards the cache for asking "how recently were this entry's last two accesses relative to each other" rather than "how many times has this entry been accessed in total." Frequency counting was the right direction early in the series but accumulated access counts conflate historical warmth with current warmth in ways IRR does not.
- Both survivors share a common insight: a single reuse since insertion provides a more useful eviction signal than total access count. SIEVE acts on "was it touched at all since insertion"; IRR acts on "how tightly clustered were the last two touches." IRR is the stricter, more informative form of the same intuition, which explains why it dominated.
- Preserving IRR timestamps across protected-to-probationary demotion replaces the halving-on-demotion mechanism naturally — a hot entry (small IRR) retains its competitive advantage in probationary without any manual counter adjustment, while cold newcomers arrive with IRR = MAX and lose the eviction contest without special handling.
- The ghost set was dropped in v003 and it won, confirming what cycles 4 through 6 suggested: ghost admission adds no value on this workload and can be omitted from the baseline going forward.
- The adaptive segment-ratio hypothesis (H01) was never evaluated due to an implementer failure. It remains untested on both the old frequency baseline and the new IRR baseline.

## Open questions

- Does IRR-based eviction transfer to non-LIRS workloads, or is it specifically aligned to this trace's generation structure? A single workload cannot distinguish broadly strong policy from a perfectly matched one.
- Is the 80/20 protected/probationary split still appropriate for IRR-based eviction, or does the sharper eviction signal change what the optimal ratio looks like? H01's adaptive balancing idea was never tested and is now more interesting on the new baseline.
- Are the current scan-window fractions (LRU quarter of protected, LRU half of probationary) still well-calibrated with IRR as the signal? IRR is a sharper discriminator than frequency counts, so narrower scan windows might suffice.

## Continuation

Next cycle should carry v003 as the new baseline: SLRU 80/20, IRR timestamps (preserved across demotion), no ghost set, LRU-quarter scan in protected, LRU-half scan in probationary. The highest-priority untested thread is adaptive segment-ratio balancing (H01 transposed onto the IRR baseline) — it was blocked this cycle and directly addresses whether the fixed split is optimal. Second priority is scan-fraction calibration: with a sharper eviction signal, narrower windows may yield equivalent quality with less scan overhead. Multi-workload evaluation is overdue and should be treated as a near-term requirement before further parameter tuning.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants probed the IRR-based SLRU baseline from cycle 7 — narrowing the probationary scan window, widening the probationary segment, and combining IRR eviction with ghost admission — and every variant regressed. The winner (v003) advances on the back of the smallest regression while confirming that the ghost-admission framework is not complementary to IRR-based eviction.

## Patterns

- The probationary half-scan is load-bearing under IRR for the same reason it was under frequency: IRR values are not uniformly MAX throughout the probationary list. Cold newcomers cluster near the LRU head, but demoted entries with modest IRR values are distributed across the second quarter too, and the scan's selection quality degrades measurably when that range is dropped.
- The 80/20 protected/probationary split is at or near the minimum required to hold this workload's active set. Widening probationary from 20% to 30% triggers excess demotion churn that IRR timestamp preservation cannot absorb — the hot working set is dense enough that a 70% protected segment is genuinely too small, mirroring the cycle-2 regression on the frequency baseline for the same structural reason.
- Ghost admission does not complement IRR eviction. The IRR signal loses the long-term access history that frequency halving preserved across demotion cycles. A node with two closely-spaced accesses far in the past is indistinguishable from a recently-bursted node under pure IRR, so combining ghost-set placement with IRR victim selection yields a worse eviction signal than either frequency-with-ghost or IRR-without-ghost alone.
- The two parameter axes that cycle 7 left open — scan fraction and segment ratio — are now settled on the IRR baseline. Both tested in the same direction as on the frequency baseline: leave them at the values inherited from the prior winning design.

## Open questions

- Does any hybrid signal that blends IRR with a visit count (or with the referenced bit) recover the long-term warmth information that pure IRR discards, without reintroducing the staleness problem of cumulative frequency counts?
- With scan fraction, segment ratio, ghost admission, and the IRR-vs-frequency signal swap all now evaluated, are there structural eviction philosophies not yet tried — such as adaptive segment balancing driven by live hit-rate feedback — that could break out of the current plateau?
- Is the IRR baseline's strength specific to the LIRS-generated trace, or does it transfer to other workloads? Multi-workload evaluation is now the clearest open question.

## Continuation

Next cycle should keep cycle/0007/v003 as the confirmed baseline (IRR timestamps preserved across demotion, 80/20 split, LRU-quarter scan in protected, LRU-half scan in probationary, no ghost set). The probationary scan fraction and segment ratio are both settled — neither narrowing nor widening is productive. Ghost admission is exhausted as a direction on the IRR baseline. The highest-value next steps are: (1) a hybrid signal that combines IRR with a coarse visit count to recover long-term warmth information, or (2) multi-workload evaluation to determine whether the current gains are broadly transferable before further parameter tuning.


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
