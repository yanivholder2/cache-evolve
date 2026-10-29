# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed ClassicLRU's per-bucket ordering. One changed where new items are inserted (v001, LIP-style insertion at len/4). One limited how far a hit is promoted (v002). One replaced the ordering with CLOCK second-chance (v003). The Winner, v001, is the only variant that changes where *new* items land. It raised hit-rate on every trace×size cell.

## Patterns

- What decides hit-rate on the MSR traces is how a new item ranks against a re-referenced item. v001 put new items below reused ones and won. v002 left new items at MRU but weakened promotion, so new items outranked reused ones, and hit-rate dropped (most on hm_0 at the larger size).
- Scan resistance matters more in the larger 1024-slot buckets. In the 128-slot buckets, v001's fixed-fraction probation window is short, so some real reuse is cut off.
- Coarser recency tracking (CLOCK) did not hurt hit-rate. It was level or slightly better, especially at the small size. Exact LRU order does not appear to be what drives hit-rate here.
- None of the variants measured per-op-time or metadata-size. Any latency claim (H03's main goal, H02's side claim, v001's cost of inserting mid-vector) is still untested.

## Open questions

- Does CLOCK (v003) actually lower get/put latency, now that we know it costs no hit-rate?
- Would a deeper insertion point, or a probation depth that scales with bucket size, recover the lost hits at 16384 without losing the gain at 131072?
- Can the two ideas be combined, i.e. CLOCK with a cold insertion position? Does graded promotion become useful once inserts land low?

## Continuation

Build on v001. Tune `INSERT_FRACTION_DIV` toward a deeper insertion point, or make probation depth scale with bucket size, focusing on the 16384 size. Make sure per-op-time is measured, and re-test CLOCK for latency, possibly combined with cold insertion. Drop bounded promotion on its own while new items still enter at MRU.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants extended cycle 0001's cold-insertion winner (v001). v001 added an absolute probation floor. v002 added BIP-style occasional MRU insertion. The third variant, v003, upgraded CLOCK to a 2-bit GCLOCK counter. The Winner is v002. Its overall gain was small and came with a split by trace: it helped hm_0, mostly at the small size, and hurt web_0.

## Patterns

- Both v001-lineage variants went after the same small-bucket shortfall, and both recovered some of it. hm_0 benefited more than web_0 each time, so hm_0 has meaningful reuse beyond the original probation window at 16384.
- Gains at the small size don't carry over to the large size. Lengthening probation left 1024-slot buckets unchanged. Sending items to MRU hurt them on web_0, where cold traffic is mostly scan-like and already drained well. A single fixed rule can't serve both trace types.
- The frequency-aware CLOCK line (v003) improved on its parent in every trace×size cell, especially on hm_0 at the large size, and it does no element shifting. It was not the Winner, but it is now a credible second lineage. Its advantage grows with bucket size, where the v001 line has the least room left to improve.
- For the second cycle in a row, no variant measured per-op-time or metadata-size. Every cost claim is still untested: mid-vector inserts, GCLOCK's multi-rotation sweeps, and the full-bucket candidate scan.

## Open questions

- Would adaptive insertion fix v002's trace split? For example, DIP-style set dueling between LIP and BIP, or an MRU-insert rate that scales with bucket size.
- Do the absolute floor (v001) and occasional MRU insert (v002) stack, or do they recover the same reuse?
- Does GCLOCK's hit-rate gain hold once its worst-case sweep cost is measured? And can it take a cold-insertion analogue, such as starting new items below already-referenced ones?

## Continuation

Build on v002, but replace its fixed period with an adaptive policy (set dueling, or a rate tied to bucket size and trace behavior) so web_0 stops losing hits. Try combining it with v001's probation floor. Keep the GCLOCK lineage (cycle 0002 v003) going as a parallel branch, since it scales well to large buckets. Make per-op-time measurement a requirement, not an option. Drop further tuning of the BIP period alone.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Two variants extended cycle 0002's bimodal-insertion winner. v001 replaced the fixed MRU-sample rate with a per-bucket PSEL feedback counter. v003 added the absolute probation floor. The third variant, v002, added ghost-ring warm admission to the GCLOCK branch. The Winner is v003, but it only moved small-bucket cells, and it moved them as a trade: web_0 gained and hm_0 lost. No variant produced a clean gain on both traces.

## Patterns

- All three variants added a short per-bucket memory (a sample ring, a ghost ring, or a fixed probation depth), and in each case that memory was sized below the reuse distance it was meant to catch. In both ring-based variants the memory forgot items before they could produce a signal. The failures were mostly about sizing, not about the ideas.
- The floor and MRU sampling do not stack. Both split the same fixed bucket space: making probation deeper shrinks the protected region where long-reuse hm_0 blocks survive. This confirms the trace split from cycle 0002 at the level of mechanism. hm_0 wants a large protected region and web_0 wants a deep probation region.
- Every change was inactive or harmful in the large buckets. The floor never engages there. The short sample ring biased PSEL toward aggressive MRU sampling, which made web_0 pollution worse. Warm admission without enough memory polluted GCLOCK. Nothing this cycle improved the large size.
- Feedback signals were miscalibrated as well as too short. PSEL rewarded any re-hit, including hits that probation insertion would have caught anyway. Warm credit added only one sweep of extra protection.
- For the third cycle in a row, per-op-time and metadata-size went unmeasured. This cycle added more unmeasured cost: linear scans on the miss path for both rings.

## Open questions

- Would true set dueling (leader buckets fixed to the v001-style depth and the v002-style depth, followers taking the winner) settle the probation/protected split per workload, where per-bucket self-feedback failed?
- Does PSEL feedback work once the tracker covers a full bucket lifetime (for example a per-item sampled bit) and credits only hits after probation?
- Is ghost admission on GCLOCK worth pursuing with a ring that covers a full rotation and a larger credit, or should that direction be dropped?

## Continuation

Pursue adaptive selection of the probation/protected split, preferably leader/follower set dueling between cycle 0002 v001's and v002's insertion depths, with no fixed combined depth. Any feedback state must be sized to the bucket's lifetime. Keep the GCLOCK branch, but try ghost admission only once more, with full-rotation coverage; drop it if that still shows no effect. Stop stacking the floor onto BIP, and require per-op-time measurement.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Two variants built on cycle 0002 v002 (LRU + bimodal insertion). v001 drove the MRU-sample rate with a PSEL counter fed by per-item tags. v003 added graded two-step promotion. The third, v002, put an S3-FIFO-style probation ring in front of the cycle 0002 GCLOCK bucket. v002 is the Winner. It improved hm_0 at both sizes and gave web_0 at the large size its biggest gain, beating that hypothesis's refutation threshold. web_0 at the small size stayed flat. This is the first cycle since 0001 where the large-bucket cells clearly moved.

## Patterns

- Cold insertion carries over to CLOCK, and it works better there than in the LRU lineage. In v002, a new item only gets into the frequency-ranked main region if it is hit while in probation, and the main sweep only runs on promotions. That filtered scans on web_0 and also kept hm_0's reused blocks protected, so it is the first change to help both traces without a trade-off. The two lineages have now traded places: GCLOCK+probation leads, and the LRU/BIP line is stuck.
- Both LRU-lineage variants traded one trace against the other again, and each failure came down to mechanism detail, not the idea itself. v001's PSEL fixed cycle 0003's lost-signal problem: per-item tags last for the item's whole lifetime. But the counter's symmetric ±1 update, starting at mid-range, votes on whether *most* samples hit, not on whether sampling pays off. So it drifted toward LIP and gave back hm_0's gain. In v003, every lower-half hit landed in one shared slot, so the protected half was cut off from the reuse stream.
- Every variant still runs into the same limit: fixed-fraction probation is too short in 128-slot buckets for web_0's late second touches. That held for the LIP lineage in cycles 0001–0003, and it now holds for v002's `len/4` ring too.
- For the fourth cycle in a row, per-op-time went unmeasured, even though all three hypotheses asked for it.

## Open questions

- Would a probation ring that is deeper in small buckets, or that adapts its size (S3-FIFO-style ghost or set dueling on `small_len`), recover web_0 at the small size without weakening the filtering in large buckets?
- Should promoted items enter main with a non-zero counter, so they are not the next sweep victim?
- Can v001's tag-based PSEL be saved with an asymmetric update, a start in the higher-sampling band, and a reward test that also counts promotions, or should the LRU/BIP line be retired?

## Continuation

Make cycle 0004 v002 (GCLOCK + probation FIFO) the main lineage. Tune or adapt `small_len`, aiming at web_0 at the small size, and try giving promotions a non-zero starting counter. At most one LRU-lineage variant should be kept: the asymmetric-PSEL fix. Drop graded promotion through a single boundary slot. Per-op-time must be reported, or the variant is not accepted.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004 v002 (GCLOCK main + probation FIFO), each testing one knob in isolation. v001 added a main-sized ghost ring for warm admission. v002 had promoted items carry a non-zero counter into main. v003 added an absolute probation-depth floor. v003 is the Winner. It was the only variant with no regressions, and it improved both traces at the small size, while the large-bucket cells stayed exactly the same as the parent.

## Patterns

- Depth helped, and both admission-strength changes hurt. Whenever the bar for entering main got lower or main got more counter mass (v001 by ghost match, v002 by carried counter), main churned more and established blocks decayed faster, mostly on web_0. Only a hit during probation is a reliable promotion signal. Only adding probation depth helped, because it gives more blocks the chance to earn that hit.
- In this lineage, the probation/protected trade-off is not zero-sum, which is the opposite of the LRU line. Main in small buckets shrank to half, and hm_0 still gained. A frequency-ranked main whose counters are decremented only on promotion needs fewer slots than an LRU protected segment. This removes the cycle 0002–0003 rule that "hm_0 wants protected and web_0 wants probation" for this design.
- Ghost admission has now failed in two lineages and at two sizings (cycle 0003 undersized, cycle 0005 main-sized). With enough coverage it brings back hm_0's near-miss reuse but re-admits web_0's single-return blocks. This is a trace split, not a sizing problem.
- The promoted block's position behind the hand already gives it a full rotation of grace. Adding a counter on top of that is redundant and harmful. This settles cycle 0004's open question with a no. v003's own takeaway still lists a non-zero promotion counter as a candidate, but v002 refuted that idea.
- The web_0 gain at the small size is real but modest, so depth explains only part of the gap. Nothing in this cohort moved the large-bucket cells upward.
- Per-op-time went unmeasured for the fifth cycle in a row. The insights say it is absent from the fitness vector itself, so this is a harness gap, not a variant omission.

## Open questions

- Does pushing the floor further (beyond half the bucket in small buckets) keep helping both traces, and where does hm_0 start to lose main capacity?
- Does a deeper probation share at the large size (such as a fraction above a quarter in 1024-slot buckets) help web_0 there, or does it weaken the filtering that cycle 0004 gained?
- What explains the rest of web_0's small-size gap, if not probation depth or admission strength?

## Continuation

Make cycle 0005 v003 (GCLOCK + probation with a depth floor) the parent. Sweep probation depth as the main lever: a higher floor in small buckets and a larger fraction in large buckets, possibly adapted with set dueling on `small_len`. Retire ghost/warm admission and non-zero promotion counters, and keep the reset-to-0 on promotion. Stop requiring per-op-time from variants until the harness adds it to the fitness vector, and flag that gap to whoever owns the harness.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005 v003 (GCLOCK main + probation FIFO with a depth floor). Two of them pushed probation depth further: v001 raised the fraction in large buckets and v002 raised the floor in small buckets. Both lost hit-rate. The third, v003, raised the main counter ceiling from 3 to 7 and is the Winner. It improved hm_0 at both sizes, most of all at the large size, which had been stuck since cycle 0004. web_0 edged up at both sizes, so nothing regressed.

## Patterns

- Probation depth is now bracketed and used up as a lever. Both depth variants failed on the trace they targeted, and the cost fell on main capacity in both cases. The "not zero-sum" result from cycle 0005 holds only up to about half the bucket in small buckets and at the existing quarter fraction in large buckets. Past those points, adding depth takes slots away from a main region that both traces need.
- The only gain came from the main side: how well main ranks its residents, not how big it is or who gets in. This is the first change since cycle 0004 to move a large-bucket cell, and it moved hm_0 without a web_0 trade-off.
- The reset-to-0 on promotion is now the key safety property of this lineage. It stopped ghost and carried-counter admission from hurting in cycle 0005, and here it stops a higher ceiling from trapping stale web_0 blocks. Any new main-side state should keep it.
- v003 reports that the gain is small and that frequency resolution is close to used up. The remaining hm_0 large-size gap (compared with cycle 0005 v001's near-miss retention) looks like a recency signal, which a counter cannot capture.
- v002's takeaway suggests a non-zero promotion counter and ghost sizing as next levers. Cycle 0005 refuted both, so those suggestions should not be followed.
- Per-op-time is still unmeasured. v003 raises the worst-case promotion sweep, so this cost is now growing without being seen.

## Open questions

- Can main add a recency signal (for example a recent-hit bit that sweeps clear before they decrement frequency) to recover hm_0's near-miss reuse without admitting web_0's single-return blocks?
- Is there a better small-bucket split strictly between 64 and 96 probation slots, or is the gain there too small to be worth pursuing?
- What still limits web_0 at the large size, now that depth, admission strength and counter resolution have all been ruled out?

## Continuation

Make cycle 0006 v003 (GCLOCK + floored probation, ceiling 7) the parent. Stop sweeping probation depth: keep the floor at 64 and the large-bucket fraction at a quarter, and allow at most one fine-grained test between 64 and 96. Focus on main-side victim selection, in particular adding recency to the frequency counter, while keeping reset-to-0 on promotion. Do not try higher ceilings, ghost admission or non-zero promotion counters again. Keep flagging the missing per-op-time measurement, now that sweep cost has grown.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0006 v003 (GCLOCK main with ceiling 7 + floored probation FIFO). v001 added a recent-hit bit and halving decay to main. v002 capped the promotion sweep and rejected promotion when main was hot. v003 made the one allowed depth test, raising the small-bucket floor between 64 and 96. v001 is the Winner. It gained most on web_0 at the large size, which had been stuck, and only just cleared the bar on its target, hm_0 at the large size. It cost a little on hm_0 at the small size. v002 gave bit-identical results to the parent, and v003 regressed both small-bucket cells.

## Patterns

- Recency decay in main helps by clearing out stale blocks, not by keeping near-miss blocks. Halving gated on recency cleared once-hot web_0 blocks faster and never hurt long-lived ones. But it barely moved hm_0, so the cycle 0006 "missing recency signal" theory explains only a small part of hm_0's large-size gap. Small buckets gained nothing, because their short main sweep already decays stale counters quickly.
- Coarser ranking costs capacity in small buckets. Bit-length ranking merges counter tiers, which undoes part of what ceiling 7 gained in the small main region. This matches the cohort-wide finding that small buckets depend on main's capacity and how finely it ranks, and they lose whenever either one shrinks.
- With reset-to-0, main always has cold victims. Every promotion puts a counter-0 block into main, and most web_0 promotions are never hit again. So the "hot main under promotion pressure" case almost never happens, which refutes H02's premise. Admission gates keyed on main being full of hot blocks are a dead end. The cycle 0006 worry about worst-case sweep cost was also mostly theoretical.
- Probation depth is closed. v003 met both refutation conditions. With ceiling 7, smaller main did not become more space-efficient, so 64 is a local optimum. This makes three cycles of evidence that probation depth is used up.
- Main-side levers now look nearly used up too: depth, admission strength, counter ceiling, admission gating and recency weighting have each been tried. The remaining hm_0 large-size gap most likely sits outside main: which blocks get promoted, or the near-miss history that cycle 0005's ghost captured.

## Open questions

- Can a small-bucket fix bring back fine tiers under the recency rule, for example ranking by recent bit plus raw counter instead of bit length, without giving up the web_0 large-size gain?
- Among main's counter-0 blocks, would choosing the victim by age or by whether it was a fresh promotion beat choosing the first one from the hand, given that main's victims are almost always fresh promotions?
- Can near-miss history be used for something other than admission (cycle 0005 refuted admission because it re-admits web_0 single-return blocks), for example as a tie-break among probation evictions?
- Did v002's evaluation really run the overlay? The code analysis predicts identical results, but a build-reuse check is worth doing once.

## Continuation

Make cycle 0007 v001 (recency bit + halving decay) the parent, and keep reset-to-0 on promotion. Next, try to recover small-bucket tier resolution under the recency rule, and look at which counter-0 block main evicts or which probation hits get promoted. Do not test probation depth again, or admission gates conditioned on main being hot. Higher ceilings, ghost admission and non-zero promotion counters stay retired. Per-op-time is still unmeasured, though v001 reduces the worst-case sweep.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007 v001 (GCLOCK ceiling 7 + recent bit + halving decay, floored probation) and kept reset-to-0. v001 used linear decay in small main regions only. v002 added a one-shot "veteran" grace bit for blocks hit in main. v003 stopped counting probation hits that arrive in a correlated-reference window as promotion credit. v003 is the Winner: it gave the lineage's largest single-cell gain since probation was added (web_0 at the large size) and nudged hm_0 up at both sizes. Its only weak cell was web_0 at the small size, which was flat to slightly down. v001 and v002 both helped and hurt nothing.

## Patterns

- For the first time, a cohort had no real regressions. All three changes were safe, and each worked on a different path: v001 on small-main decay, v002 on the order of main's counter-0 victims, v003 on the probation promotion signal. They do not overlap, so they are candidates to combine, not alternatives.
- The levers depend on scale, and each acts at only one size. v001 changed only the 64-slot main (the large cells were bit-identical). v002 and v003 mattered mainly in large buckets, because the grace visit and the window are both too short to matter in small ones. Changes should be sized in absolute slots per region, not as one rule or ratio for every size.
- Tightening admission keeps winning. v003 is the fourth tightening of who reaches main to succeed (after probation, the depth floor and reset-to-0). It is also the first change to the promotion signal itself that worked. Burst re-references on web_0 were a meaningful share of the wasted counter-0 promotions that cycle 0007 identified.
- web_0 gains more from main-side history than expected. v002's veteran bit helped web_0 at the large size more than hm_0, which refutes the idea that web_0's decayed blocks are mostly stale.
- hm_0's large-size gap is still mostly unexplained. Recency (cycle 0007), victim ordering (v002) and promotion filtering (v003) each moved it only a little. Ranking inside main now looks nearly used up for hm_0.
- Small buckets reward finer ranking. v001's recent bit plus linear counter confirmed cycle 0007's tier-resolution diagnosis without hurting web_0. Note that v002's insight blames the small-bucket cost on "v001", but it means cycle 0007 v001, not this cycle's v001.

## Open questions

- Does widening v003's window in 128-slot buckets (for example an absolute floor of about 16 slots) bring the web_0 gain to the small size, and at what width does hm_0's real reuse start being filtered?
- Do v001, v002 and v003 stack? They act on different paths and sizes, but v002's grace bit and v003's filter both target counter-0 promotions in large main regions, so their gains may overlap.
- What drives hm_0's remaining large-size gap? It is likely long-distance reuse that probation never sees.

## Continuation

Make cycle 0008 v003 the parent. First try folding in v001's size-gated linear decay, which should be close to free because it touches only small buckets. Also widen the correlated-reference window in small buckets using an absolute floor. Test v002's veteran bit on top of v003 as a separate variant, because the two may overlap. Do not revisit probation depth, hot-main admission gates, higher ceilings, ghost admission or non-zero promotion counters. Per-op-time is still unmeasured.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008 v003 (GCLOCK ceiling 7 + recent bit + halving, floored probation, correlated-reference filter, reset-to-0) and tested the three combinations that cycle 0008 proposed. v001 folded in linear decay, gated to small main regions, and is the Winner. It lifted both 16384 cells, including web_0 at the small size, which had been v003's weak cell, and left the 131072 cells bit-identical. v003 (the veteran grace bit on top of the filter) helped a little everywhere and hurt nothing. v002 (a wider small-bucket window) traded web_0 for hm_0 at the small size and was not confirmed.

## Patterns

- The size-gated split now works as intended. Small buckets want fine linear tiers and large buckets want halving. Under the filter, linear decay gained more than it did on its own: once burst-only blocks are kept out of main, the finer tiers are spent ranking blocks that really are reused.
- The two changes aimed at counter-0 promotions overlap. The veteran grace still adds something on top of the filter, but only part of what it gained alone, because the filter has already removed many of the fresh promotions the grace used to outrank. It still costs nothing, though, so it is worth keeping.
- The correlated-reference window does not scale as an absolute width. Sixteen slots is a small fraction of the large probation ring but a quarter of the small one, and at that width it starts to filter hm_0's real short-gap reuse. The window is close to what it can do in small buckets.
- Small-bucket web_0 is no longer limited mainly by bursts. The lever that moved it this cycle was main-side ranking (linear tiers), not the promotion filter.
- Large buckets moved only through v003's grace. The large-size hm_0 gap is still largely unexplained.

## Open questions

- Does v003's veteran bit stack with v001's linear decay? In small buckets both act on main ranking, so the grace's small-size gain may vanish. Its large-size gain should survive, because v001 does not touch large buckets.
- Is there a small-bucket window strictly between the current narrow one and v002's quarter-ring width that filters web_0 bursts without catching hm_0 reuse, or is the window used up at that size?
- What still limits hm_0 at the large size? Main ranking, recency and promotion filtering have each moved it only a little.

## Continuation

Make cycle 0009 v001 the parent, and first add v003's veteran bit on top of it. Expect gains mainly at 131072, and check whether the small-size gain survives under linear tiers. Stop widening the small-bucket correlated window. At most, test one intermediate width, and do not use an absolute 16. Keep reset-to-0. Probation depth, hot-main admission gates, higher ceilings, ghost admission and non-zero promotion credit all stay retired. Look for a new lever for large-size hm_0 rather than tuning main ranking further. Per-op-time is still unmeasured, and linear decay brings back a longer worst-case sweep in small buckets.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009 v001 (GCLOCK ceiling 7, size-gated linear/halving decay, floored probation, correlated-reference filter, reset-to-0). v001 stacked the veteran grace bit. v002 turned the probation FIFO into a CLOCK ring that gives recent-only slots one extra lap. v003 widened the correlated window in large buckets only. v002 is the Winner. It gave a small hm_0 gain at the small size and held or slightly raised web_0 at both sizes, but it dipped hm_0 at the large size, so its own claim is only half confirmed. v001 got the large-size gains it predicted but cost hm_0 at the small size, landing right on the tolerance bound. v003 regressed both large cells.

## Patterns

- Every lever in this lineage works at only one bucket size. Linear decay pays off only in small mains and the veteran bit only in large halving mains. v002's lap rule helps where the window is narrow (small rings) and hurts where it is wide (large rings). The cycle 0008 finding that changes should be sized per region now holds for every lever. Each new mechanism should come with a size gate.
- The veteran bit and linear tiers collide in small mains. In large buckets, v001 matched the veteran bit's standalone gain exactly, so it stacks cleanly there. In small mains it blurs the fine tiers. Gating it to halving buckets should keep the gain and cost nothing.
- Probation retention is zero-sum. In a position-based ring, each extra lap for one slot shortens probation for the slots behind it. The net sign depends on how wide the window is compared with the ring. This is a different version of "probation depth is closed", not a way around it.
- Window width is now used up at both sizes. Widening the large-bucket window filtered web_0's real early reuse, just as cycle 0009 v002 filtered hm_0's in small buckets. About 16 slots is the most either size tolerates. The window grows its gain only up to that point.
- hm_0 at the large size is still the unexplained gap. This cycle, every change touching it was neutral or slightly negative except the veteran bit.

## Open questions

- Do v002's lap rule (gated to small buckets) and v001's veteran bit (gated to halving buckets) combine cleanly? On paper they act on disjoint sizes and disjoint regions.
- Would a narrower large-bucket window, below 16 slots, recover any of web_0's early reuse without bringing bursts back?
- Is the large-size hm_0 gap driven by long-distance reuse that any probation-side lever is structurally unable to see?

## Continuation

Make cycle 0010 v002 the parent and gate its one-lap rule to small (non-halving) buckets. Then add the veteran bit gated to halving buckets. Expect both to keep their gains at their own size with no cross-size cost. Stop widening the correlated window at any size. At most, test one narrower large-bucket width. Keep reset-to-0. Probation depth, hot-main admission gates, higher ceilings, ghost admission and non-zero promotion credit stay retired. Large-size hm_0 still needs a new kind of lever, not more probation or main-ranking tuning. Per-op-time is still unmeasured, and v002 adds a bounded probation walk per miss.


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
