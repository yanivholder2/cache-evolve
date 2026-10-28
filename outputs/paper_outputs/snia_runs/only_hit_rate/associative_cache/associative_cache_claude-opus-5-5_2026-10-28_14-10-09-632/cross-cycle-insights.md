# Cross-Cycle Insights (cycles 0001–0010)

## Recurring wins

- **Ranking new items below reused ones.** Cold or probation insertion won in cycle 0001 (LIP on LRU) and again in cycle 0004 (a probation FIFO in front of GCLOCK). Every later Winner keeps it. On the MSR traces, the main thing that decides hit-rate is how a new item ranks against a re-referenced one.
- **Making admission to main stricter.** Each successful step made it harder to reach main: probation (0004), the depth floor (0005), reset-to-0 on promotion (0005–0006), and the correlated-reference filter (0008). A hit during probation, outside a burst, is the only promotion signal that has held up.
- **Reset-to-0 on promotion is the lineage's key safety property.** It kept ghost admission, carried counters, higher ceilings and hot-main gates from doing more damage. It also guarantees main always has cold victims. Keep it in every variant.
- **Improving how main ranks its residents, not how big main is.** The changes that helped were ceiling 7 (0006), the recent bit with halving decay (0007), linear tiers in small mains (0008–0009), and the veteran grace bit in large mains (0008–0010). These were the only changes that moved large-bucket cells after cycle 0004.
- **Gating each lever to one bucket size.** Since cycle 0008, every lever has acted at only one size. Small mains want fine linear tiers. Large halving mains want recency and veteran grace. Changes defined in absolute slots per region beat one fixed ratio applied to every size.
- **Combining changes that act on separate paths.** Changes that touched different paths stacked roughly additively, for example linear decay with the filter in cycle 0009. Changes aimed at the same population, the counter-0 promotions, overlapped only partly.

## Recurring failures

- **Fixed rules applied to both traces.** hm_0 and web_0 have kept pulling in opposite directions. BIP, the floor plus BIP, and PSEL all traded one trace for the other (cycles 0002–0004).
- **Per-bucket self-feedback (PSEL) and short sample or ghost rings.** These failed in cycles 0003 and 0004. The state was sized below the reuse distance, and the reward signal was miscalibrated: it was symmetric and credited hits that probation would have caught anyway. The LRU/BIP lineage was effectively retired after cycle 0004.
- **Ghost or warm admission.** This failed in two lineages and at two sizings (cycles 0003 and 0005). It brings back hm_0's near-miss blocks but re-admits web_0's single-return blocks. That is a trace split, not a sizing problem.
- **Non-zero promotion counters and higher ceilings.** A promoted block already gets a lap of grace from its position behind the hand. Adding counter mass on top makes main churn more (cycles 0005 and 0006).
- **Probation depth or retention as a lever.** It is bracketed and closed. Small buckets peak at a floor of about 64 and large buckets at about a quarter (cycles 0006 and 0007). Retention through a position ring is zero-sum (cycle 0010).
- **Widening the correlated-reference window.** About 16 slots is the maximum at either size. Going wider filters real early reuse: hm_0 in small buckets (cycle 0009) and web_0 in large buckets (cycle 0010).
- **Admission gates keyed on main being "hot".** The premise never occurs under reset-to-0, so the change ran inertly with no effect (cycle 0007).
- **Graded or bounded promotion while new items still enter at MRU,** and **graded promotion through a single boundary slot.** Both lost hit-rate (cycles 0001 and 0004).

## Open questions

- **What drives the large-bucket hm_0 gap?** It has been the main unexplained gap for about six cycles. Recency, victim order, promotion filtering and lap retention each moved it only a little, apart from the veteran bit. It is likely long-distance reuse that no probation-side or main-ranking lever can see, so a new kind of lever is needed.
- **Do the cycle 0010 lap rule (gated to small buckets) and the veteran bit (gated to halving buckets) combine cleanly** at their own sizes, with no cost at the other size?
- **Would a large-bucket correlated window narrower than 16 slots** recover web_0's early reuse without letting bursts back in?
- **Can near-miss history serve some role other than admission,** for example as a tie-break among probation evictions?
- **Per-op-time and metadata-size have never been measured in ten cycles.** They are absent from the fitness vector. Unmeasured costs keep growing: the longer worst-case sweeps from linear decay, and the bounded probation walk per miss. This is a harness gap.

## Per-policy notes

- **LRU / LIP / BIP (cycles 0001–0004):**
  - LIP was the first win.
  - The probation and protected regions compete for one fixed space, so the policy is zero-sum: hm_0 wants a large protected region and web_0 wants deep probation.
  - The floor and BIP do not stack.
  - PSEL drifted toward LIP.
  - The lineage has been dormant since cycle 0004.
- **CLOCK / GCLOCK, standalone (cycles 0001–0003):**
  - Coarse recency cost no hit-rate.
  - A 2-bit GCLOCK counter improved on CLOCK in every cell, and the gain was largest in large buckets.
  - Ghost admission on top of it failed.
- **GCLOCK main + probation FIFO (cycles 0004–0010, the current lineage):**
  - Probation and main are not zero-sum here, up to the depth bracket. A frequency-ranked main needs fewer slots than an LRU protected segment.
  - The current stack is:
    - ceiling 7
    - a recent bit
    - size-gated decay: linear in small mains, halving in large mains
    - a floored probation FIFO
    - a correlated-reference filter
    - reset-to-0 on promotion
  - The cycle 0010 parent adds a one-lap CLOCK rule in probation. This rule helps in small rings and hurts in large rings, so it needs a size gate.
  - The veteran bit stacks in large mains but blurs the linear tiers in small mains, so it should be gated to halving buckets only.
