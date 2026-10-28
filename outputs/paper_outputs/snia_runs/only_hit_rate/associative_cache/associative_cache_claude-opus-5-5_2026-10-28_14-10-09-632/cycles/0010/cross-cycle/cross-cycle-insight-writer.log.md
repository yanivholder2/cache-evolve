# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:19:29+00:00

## timeline

### 18:19:52 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0010)\n\n## Recurring wins\n\n- **Ranking new items below reused ones.** Cold or probation insertion won in cycle 0001 (LIP on LRU) and again in cycle 0004 (a probation FIFO in front of GCLOCK). Every later Winner keeps it. On the MSR traces, the main thing that decides hit-rate is how a new item ranks against a re-referenced one.\n- **Making admission to main stricter.** Each successful step made it harder to reach main: probation (0004), the depth floor (0005), reset-to-0 on promotion (0005–0006), and the correlated-reference filter (0008). A hit during probation, outside a burst, is the only promotion signal that has held up.\n- **Reset-to-0 on promotion is the lineage's key safety property.** It kept ghost admission, carried counters, higher ceilings and hot-main gates from doing more damage. It also guarantees main always has cold victims. Keep it in every variant.\n- **Improving how main ranks its residents, not how big main is.** The changes that helped were ceiling 7 (0006), the recent bit with halving decay (0007), linear tiers in small mains (0008–0009), and the veteran grace bit in large mains (0008–0010). These were the only changes that moved large-bucket cells after cycle 0004.\n- **Gating each lever to one bucket size.** Since cycle 0008, every lever has acted at only one size. Small mains want fine linear tiers. Large halving mains want recency and veteran grace. Changes defined in absolute slots per region beat one fixed ratio applied to every size.\n- **Combining changes that act on separate paths.** Changes that touched different paths stacked roughly additively, for example linear decay with the filter in cycle 0009. Changes aimed at the same population, the counter-0 promotions, overlapped only partly.\n\n## Recurring failures\n\n- **Fixed rules applied to both traces.** hm_0 and web_0 have kept pulling in opposite directions. BIP, the floor plus BIP, and PSEL all traded one trace for the other (cycles 0002–0004).\n- **Per-bucket self-feedback (PSEL) and short sample or ghost rings.** These failed in cycles 0003 and 0004. The state was sized below the reuse distance, and the reward signal was miscalibrated: it was symmetric and credited hits that probation would have caught anyway. The LRU/BIP lineage was effectively retired after cycle 0004.\n- **Ghost or warm admission.** This failed in two lineages and at two sizings (cycles 0003 and 0005). It brings back hm_0's near-miss blocks but re-admits web_0's single-return blocks. That is a trace split, not a sizing problem.\n- **Non-zero promotion counters and higher ceilings.** A promoted block already gets a lap of grace from its position behind the hand. Adding counter mass on top makes main churn more (cycles 0005 and 0006).\n- **Probation depth or retention as a lever.** It is bracketed and closed. Small buckets peak at a floor of about 64 and large buckets at about a quarter (cycles 0006 and 0007). Retention through a position ring is zero-sum (cycle 0010).\n- **Widening the correlated-reference window.** About 16 slots is the maximum at either size. Going wider filters real early reuse: hm_0 in small buckets (cycle 0009) and web_0 in large buckets (cycle 0010).\n- **Admission gates keyed on main being \"hot\".** The premise never occurs under reset-to-0, so the change ran inertly with no effect (cycle 0007).\n- **Graded or bounded promotion while new items still enter at MRU,** and **graded promotion through a single boundary slot.** Both lost hit-rate (cycles 0001 and 0004).\n\n## Open questions\n\n- **What drives the large-bucket hm_0 gap?** It has been the main unexplained gap for about six cycles. Recency, victim order, promotion filtering and lap retention each moved it only a little, apart from the veteran bit. It is likely long-distance reuse that no probation-side or main-ranking lever can see, so a new kind of lever is needed.\n- **Do the cycle 0010 lap rule (gated to small buckets) and the veteran bit (gated to halving buckets) combine cleanly** at their own sizes, with no cost at the other size?\n- **Would a large-bucket correlated window narrower than 16 slots** recover web_0's early reuse without letting bursts back in?\n- **Can near-miss history serve some role other than admission,** for example as a tie-break among probation evictions?\n- **Per-op-time and metadata-size have never been measured in ten cycles.** They are absent from the fitness vector. Unmeasured costs keep growing: the longer worst-case sweeps from linear decay, and the bounded probation walk per miss. This is a harness gap.\n\n## Per-policy notes\n\n- **LRU / LIP / BIP (cycles 0001–0004):**\n  - LIP was the first win.\n  - The probation and protected regions compete for one fixed space, so the policy is zero-sum: hm_0 wants a large protected region and web_0 wants deep probation.\n  - The floor and BIP do not stack.\n  - PSEL drifted toward LIP.\n  - The lineage has been dormant since cycle 0004.\n- **CLOCK / GCLOCK, standalone (cycles 0001–0003):**\n  - Coarse recency cost no hit-rate.\n  - A 2-bit GCLOCK counter improved on CLOCK in every cell, and the gain was largest in large buckets.\n  - Ghost admission on top of it failed.\n- **GCLOCK main + probation FIFO (cycles 0004–0010, the current lineage):**\n  - Probation and main are not zero-sum here, up to the depth bracket. A frequency-ranked main needs fewer slots than an LRU protected segment.\n  - The current stack is:\n    - ceiling 7\n    - a recent bit\n    - size-gated decay: linear in small mains, halving in large mains\n    - a floored probation FIFO\n    - a correlated-reference filter\n    - reset-to-0 on promotion\n  - The cycle 0010 parent adds a one-lap CLOCK rule in probation. This rule helps in small rings and hurts in large rings, so it needs a size gate.\n  - The veteran bit stacks in large mains but blurs the linear tiers in small mains, so it should be gated to halving buckets only.\n"
}
```

### 18:19:59 chat

I wrote `cross-cycle-insights.md` in the cycle 0010 cross-cycle folder. It has the four required sections, built only from the ledger digest for cycles 0001–0010.

- **Recurring wins:**
  - Putting new items below reused ones, through cold or probation insertion, keeps winning.
  - Each step that made it harder to get into main paid off.
  - Reset-to-0 on promotion is what keeps the current line safe.
  - The later gains came from how main ranks blocks, not from how big main is.
  - Since cycle 0008, every change has worked at only one bucket size, so each needs a size gate.
- **Recurring failures:**
  - Fixed rules that trade hm_0 against web_0.
  - PSEL feedback and short history rings.
  - Ghost admission, which failed in two lineages.
  - Non-zero promotion counters and higher counter ceilings.
  - Probation depth, which is now closed.
  - Widening the burst-filter window past about 16 slots.
  - Admission gates that wait for main to be "hot", which never happens under reset-to-0.
- **Open questions:**
  - The large-bucket hm_0 gap has gone unexplained for about six cycles and probably needs a new kind of change.
  - Whether the two size-gated changes from cycle 0010 combine cleanly.
  - Whether a narrower burst-filter window helps in large buckets.
  - Whether near-miss history can be used for something other than admission.
  - Per-op-time and metadata size have not been measured once in ten cycles. That is a gap in the test harness, not in the variants.
- **Per-policy notes:** I split these into three groups: the LRU line (cycles 0001–0004, inactive since), standalone CLOCK/GCLOCK (0001–0003), and the current GCLOCK-plus-probation line (0004–0010), including what the cycle 0010 parent contains.


---

- finished: 2026-10-28T18:19:59+00:00
- duration: 30.63s
- status: ok
- input_tokens: 4
- output_tokens: 2942
- cache_read_tokens: 44727
- cache_write_tokens: 17103
- cost_usd: 0.204625
