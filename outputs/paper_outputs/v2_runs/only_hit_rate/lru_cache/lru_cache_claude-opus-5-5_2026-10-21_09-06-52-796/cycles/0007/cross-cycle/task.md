# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants branched from the baseline:
- **v001:** an admission-filtering policy (S3-FIFO).
- **v002:** a cheaper LRU approximation (CLOCK).
- **v003:** a representation-only control that keeps strict LRU and adds Fx hashing and `u32` links.

v001 won by raising hit rate on the scan- and loop-heavy workload-a. v002 roughly matched LRU's hit rate, and v003 left it unchanged, as designed.

## Patterns

- **Only hit rate was measured.** None of the variants reported `metadata-size` or `per-op-time`. Two of the three hypotheses (H02, H03) were aimed mainly at those dimensions, so their main claims are still untested.
- **Admission filtering is what helped.** On this trace, the gain came from changing which keys get admitted. Changing how recency is tracked did not help.
- **Exact LRU ordering adds nothing to hit rate here.** CLOCK's coarser second-chance recency held hit rate at LRU's level or slightly above. Precise LRU order can be given up without a hit-rate penalty.
- **All three variants remove or shrink the `prev`/`next` links.** Each predicts a smaller node and cheaper hits, but none has been measured. v001 also adds ghost structures stored outside `Node`, which could cancel its memory savings.

## Open questions

- Does v001's ghost queue and set add more per-key memory than dropping the links saves? And is its `get` actually faster?
- Does CLOCK's victim sweep cause tail-latency spikes on `put` when most entries are marked referenced?
- Are v003's representation changes (Fx hasher, `u32` links, MRU fast path) independent of policy? If so, they could be stacked onto v001 or v002 for a speed gain.

## Continuation

Re-measure `metadata-size` and `per-op-time` for v001, v002 and v003 before building on them. If the harness can't produce these dimensions, fix that first. Then use v001 as the parent and try:
- shrinking or compacting its ghost structures,
- grafting v003's hasher and narrow indices onto it,
- tuning the size of the small probation queue.

Don't spend further cycles on CLOCK-style recency tweaks unless measurement shows a clear memory or latency win.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants changed cycle 1's S3-FIFO: v001 added a compact fingerprint ghost with a longer window, and v002 made the small queue adaptive in the style of ARC. A third, v003, built LIRS from scratch on the baseline. v001 was picked as the Winner, but it is only the best of a weak cohort. Both S3-FIFO changes lowered hit rate compared with their parent. LIRS beat LRU but did not reach the S3-FIFO hit rate.

## Patterns

- **Hit rate is still the only thing measured.** For the third hypothesis in a row, a claimed memory or speed win (v001's fingerprint ring) cannot be checked. LIRS's likely costs (a larger `Node`, a linear walk to enforce the ghost bound) cannot be checked either. The harness gap flagged in cycle 1 is still open.
- **Every change that made admission looser lowered hit rate.** v001's longer ghost window and v002's main-ghost fast path both let keys into `main` without proving reuse while they were resident. v002's growing `small` queue also exposed more of the cache to scans. Cycle 1 credited S3-FIFO's gain to strict admission filtering. This cycle supports that from the other side: weakening it costs hit rate.
- **A ghost hit does not mean the key will hit if kept.** On loops longer than the cache, a key can be re-referenced within the ghost window yet not within `main`'s residency. Lengthening the ghost window (v001) or treating ghost hits as a signal to resize (v002) both acted on this misleading signal.
- **The fixed ~10% probation split looks close to right.** v002 drifted away from it and lost hit rate. This answers cycle 1's question about sensitivity: hit rate depends on the split.
- **LIRS's idea of a protected set works, but in this form it is heavier and no better** than the simpler S3-FIFO filter on this trace.

## Open questions

- With the ghost window kept at the parent's size, does the fingerprint ring alone actually reduce `metadata-size` and `per-op-time`?
- Would LIRS with a larger HIR fraction and an O(1) non-resident FIFO close the gap to S3-FIFO, and what would it cost in memory?
- Are v001's stale-slot and early-decrement issues in the ring large enough to matter once the window is back to its original size?

## Continuation

Fix measurement of `metadata-size` and `per-op-time` before anything else. Further hit-rate-only cycles cannot judge the memory and speed claims that most hypotheses now make. Use cycle 1's v001 as the parent, not this cycle's. Then:
- keep the fixed ~10% probation split,
- try the fingerprint ring at the original ghost size, so the representation change is tested separately from the policy change,
- graft on cycle 1 v003's hasher and narrow indices.

Drop ARC-style adaptive sizing, longer ghost windows, and any other path that admits keys into `main` without proven reuse. Return to LIRS only once its cost can be measured and the linear bound-walk is fixed.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants changed a single policy knob. Two of them branched from cycle 1's S3-FIFO:
- **v001** shrank the ghost window to half of capacity.
- **v002** raised the probation promotion bar to two hits.

The third, **v003**, enlarged LIRS's resident-HIR queue. v001 won with a clear hit-rate gain. v002 was essentially flat, and v003 regressed slightly.

## Patterns

- **The ghost is the main lever on admission.** Cycle 2 lengthened the ghost and lost hit rate. This cycle shortened it and gained. Ghost size is a real tuning knob, and the parent's near-capacity ghost was too loose.
- **Probation strictness and ghost size can't be tuned separately.** With a large ghost, a key that v002 rejects from `small` comes back through the ghost-hit path straight into `main`. Its stricter bar only delays admission by one miss. Any test of "stricter admission" has to change the ghost path too.
- **Probation size doesn't explain why S3-FIFO beats LIRS.** Giving LIRS a bigger probation buffer cost protected LIR slots and didn't help. The gap between the two policies comes from somewhere else, most likely how each one handles ghost and non-resident re-admission.
- **Measurement is still missing.** For the third cycle in a row, only hit rate was reported. v001's smaller ghost probably saves memory, but that is unverified. Cycle 2's planned representation tests (fingerprint ring at the original ghost size, the hasher and narrow-index graft) were not attempted.

## Open questions

- Where is the optimum ghost size? Would going below half of capacity start dropping genuine re-references?
- Would sending ghost hits back into `small` instead of `main` beat simply shrinking the ghost? Does it make a stricter promotion bar meaningful?
- How much `metadata-size` does the smaller ghost actually save?

## Continuation

Use this cycle's v001 as the parent. Keep searching ghost size, going smaller (for example a quarter of capacity). Separately, test sending ghost hits into `small` rather than `main`. Revisit `PROMOTE_FREQ` only together with one of those ghost changes.

Drop the LIRS HIR-ratio tuning: restore the 1% split, or set LIRS aside. Measuring `metadata-size` and `per-op-time` remains the top harness priority. Once a ghost-size optimum is settled, run the representation-only experiments that cycle 2 asked for.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants came from cycle 3's v001 (ghost at half of capacity) and each tried a different way to filter the ghost-hit path into `main`:
- **v001** shrank the ghost again, to a quarter of capacity. It won with a modest gain.
- **v003** kept the ghost size and added a count-min frequency gate on ghost re-admission. Its gain was only marginal.
- **v002** routed ghost hits into `small` and required two hits for cold keys. Its hit rate regressed.

## Patterns

- **Shrinking the ghost is still the best lever, but the gains are getting smaller.** Two halvings in a row have helped, and this one helped less than the last. The optimum is probably close to a quarter of capacity or a little below it. It has only been measured at one cache size.
- **A ghost hit is a useful reuse signal once the window is short.** Cycle 2 called ghost hits misleading on long loops. With a short window, though, they mostly catch real reuse. v002 made ghost-hit keys prove reuse again inside the tiny probation queue and lost hit rate. v003's frequency gate found little left to filter.
- **Requiring reuse while resident has hit a limit.** In earlier cycles, stricter admission kept paying off. Here it stopped: with a probation queue of about 10%, a two-hit bar or a forced pass through `small` evicts keys whose reuse gap is only slightly longer than that queue. Admission can be too strict as well as too loose.
- **Gates on the ghost path leak.** Both v002 and v003 left keys a second route into `main` through `small`, the same leak cycle 3 found with the promotion bar. v003 also compared against `main.front()`, which is often not the entry that actually gets evicted. Any admission gate should compare against the real eviction victim.
- **Measurement is still missing, for the fourth cycle running.** v001's smaller ghost probably saves memory. v003's sketch probably costs both memory and time, since it adds a table and extra hashing on every operation. Neither effect has been verified.

## Open questions

- Where does hit rate turn over: an eighth of capacity, or lower? Does the best ghost size hold at other cache sizes?
- Was v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately. Its ghost bookkeeping fix, which stops stale entries erasing newer ones, is correct and should be kept.
- Would TinyLFU-style admission against the real eviction victim, applied to all insertions and not only ghost hits, pay back the sketch's unmeasured cost?

## Continuation

Use this cycle's v001 as the parent. Run one more ghost-size probe at an eighth of capacity, at more than one cache size if possible, then freeze the ghost size. Port v002's sequence-number ghost fix by itself as a correctness change.

Drop forced routing of ghost hits through `small`, and drop two-hit promotion bars for now. Drop ghost-path-only frequency gates. Bring the sketch back only as victim-based TinyLFU admission, and only once `per-op-time` and `metadata-size` can be measured. With the ghost size settling, run the long-postponed representation-only experiments next: the fingerprint ring and the hasher plus narrow-index graft. Getting the harness to measure those two dimensions is still the top priority.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants came from cycle 4's v001 (ghost at a quarter of capacity). Each ran one of the experiments cycle 4 asked for:
- **v001** halved the ghost again, to an eighth of capacity. It won, with a smaller gain than the previous halving.
- **v002** swapped the ghost to a fingerprint ring at the same window size, changing only the representation. Hit rate was exactly unchanged.
- **v003** added TinyLFU admission on every route into `main`, compared against the real CLOCK victim. Its gain was only marginal.

## Patterns

- **Shrinking the ghost still pays, but by less each time.** Three halvings in a row have helped, each less than the one before. The ghost window is now about the size of the probation queue, so the next halving would make the ghost smaller than `small`. That probe really asks whether the ghost-into-`main` fast path is worth having at all. Every result so far comes from one trace at one cache size.
- **Frequency admission is near its ceiling on this design.** v003 fixed both flaws found in cycle 4: the leak through `small` and comparing against the wrong entry. It still gained only a hair. The small-queue filter plus a short ghost already turns away most of the low-frequency candidates that TinyLFU is meant to block. So the leak and the wrong target were not what held the sketch back.
- **The representation track finally ran, and as expected it left hit rate alone.** v002 confirms the fingerprint ring is a safe base for later ghost work. The stale-erase fix made no measurable difference, because the case it guards against almost never happens. Whether the ring is a net win isn't known yet: it removes key clones but adds an extra hash pass on each miss.
- **Measurement is still missing, for the fifth cycle running.** This cycle makes the gap costly. One variant's only claimed benefit sits entirely in the unmeasured dimensions. Another adds a per-access sketch whose memory and time cost could outweigh its tiny hit-rate gain. Neither can be judged.

## Open questions

- At a sixteenth of capacity (ghost smaller than `small`), does hit rate finally turn over, or does the ghost path turn out to be dispensable?
- Do the ghost-size gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances?
- Does the fingerprint ring actually cut `metadata-size` and `per-op-time`, given its extra hashing? Is the sketch's per-access cost close to zero?

## Continuation

Use this cycle's v001 as the parent. Run the sixteenth-of-capacity probe, and if at all possible run it at a second cache size as well, before trusting further tiny ghost-size gains. Port v002's fingerprint ring onto that parent as a policy-neutral base. Keep its hashing cost in mind: one hash per key, used for both the fingerprint and the lookup, if that can be arranged.

Drop TinyLFU and other frequency-admission gates unless cost measurements show they are nearly free. Look for hit-rate gains outside admission filtering. Measuring `metadata-size` and `per-op-time` is now blocking: without it, the representation and sketch results can't be judged.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants came from cycle 5's v001 (ghost at an eighth of capacity, `small` at a tenth). Each changed one knob:
- **v001** halved the ghost to a sixteenth of capacity. Hit rate was flat.
- **v002** widened `main`'s frequency cap from 3 to 7. It gained a little.
- **v003** halved `small` to a twentieth of capacity and kept the ghost at an eighth. It won.

All the gains were small.

## Patterns

- **Ghost and `small` act as one admission window.** v001 and v003 point the same way. With `small` at a tenth of capacity, a ghost smaller than `small` adds almost nothing. Once `small` shrinks, a ghost larger than `small` does real work as a cheap second stage of probation. It costs a ghost key, not a resident slot, and the freed slots go to `main`. The question is no longer "how small can the ghost be". It is "how should the combined window be split between resident probation and ghost".
- **Ghost-only tuning is done.** The ghost-size curve has gone flat. Taken alone, v001 suggests the fast path is dispensable. v003 shows that conclusion holds only while `small` stays large.
- **Retention is a separate lever, and it barely moves.** v002 is the first winner-grade result outside admission. Its gain is about as small as the last ghost halving. Telling moderately hot keys apart from very hot keys matters little on this trace. It is orthogonal to v003's change, though, so the two could stack.
- **The evidence base is getting thin.** Every gain this cycle is small, and all of them come from one trace at one cache size. Results at a second size were asked for in cycles 4 and 5 and still haven't come. v003's in-file tests don't exercise the new ratio at their tiny capacities. The fingerprint-ring port that cycle 5 asked for was not attempted. `metadata-size` and `per-op-time` are unmeasured for the sixth cycle running.

## Open questions

- With `small` at a twentieth, what ghost size is best? Is the right rule to keep `small + ghost` roughly constant while moving slots from `small` to the ghost?
- How far can `small` shrink before keys that arrive in `main` with no frequency credit start to hurt?
- Does v002's wider cap stack with v003's split, or do the two overlap?
- Does any of this hold at a second cache size?

## Continuation

Use this cycle's v003 as the parent. Probe the `small`/ghost split together: for example, `small` at a fortieth, or the ghost moved back up while `small` stays at a twentieth. Separately, port v002's `MAX_FREQ = 7` onto v003 to test whether the two stack. Add tests at capacities large enough to exercise the new ratio.

Stop probing ghost size on its own. Before chasing more gains this small, get a second cache size measured. Still pending: porting the fingerprint ring, and measuring `metadata-size` and `per-op-time`.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants came from cycle 6's v003 (`small` at a twentieth of capacity, ghost at an eighth). Each tested one follow-up that cycle 6 had asked for:
- **v002** shrank `small` to a fortieth and grew the ghost so that `small + ghost` stayed the same size. It won, by a marginal amount.
- **v001** ported `MAX_FREQ = 7` onto the parent. Its gain was marginal, so the two levers overlap rather than stack.
- **v003** started ghost-admitted keys at `freq = 1`. Hit rate went down slightly.

## Patterns

- **Holding the combined window constant still works, but the gains are shrinking.** Moving resident probation slots into the ghost helped again, by less than last cycle. This repeats the pattern seen with ghost-only halvings. Two forces work against the extra space in `main`. `small` now catches less reuse while keys are resident. More keys also reach `main` through the ghost with no credit.
- **Retention changes in `main` are close to used up.** A higher frequency cap adds little once `main` is large. The reason is that credits only build up while a key is in `main`, and both admission paths reset them to zero.
- **Starting credits have to respect how strong the evidence is.** A ghost hit shows two references, and none of them happened while the key was resident. Giving these keys a free credit ranked them above `small` promotions, which did show a hit while resident. `main` then held on to false positives for longer. With a short window, `main`'s CLOCK loop already protects keys that really are reused. This contradicts the parent's idea that ghost arrivals "lack protection".
- **The cohort pattern is the same as in earlier cycles: three small, single-knob moves, all measured on one trace at one cache size.** A second cache size, the fingerprint-ring port, and measurement of `metadata-size` and `per-op-time` are still missing, for the seventh cycle running.

## Open questions

- Does another shift from `small` to the ghost still help, or is `small` at a fortieth the floor? Does the split rule hold at a second cache size?
- Would starting credits that are consistent across both admission paths help? For example, give a matching credit to `small` promotions, or give a credit only on a repeat ghost hit.
- If credits carried over from `small` into `main` instead of resetting, would the higher frequency cap start to pay off?

## Continuation

Use this cycle's v002 as the parent, and keep `MAX_FREQ = 7` on it since it costs nothing. Before shifting any more slots from `small` to the ghost, run the current split at a second cache size. The admission-window line of work now produces gains too small to trust from a single size.

Drop raising the frequency cap again, and drop starting credits that apply to only one admission path. If credit work continues, change how credits are earned or carried over, and treat both admission paths the same way. Still pending: the fingerprint-ring port, and measuring `metadata-size` and `per-op-time`.


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
