# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-28T15:27:27+00:00

## timeline

### 15:27:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0008)\n\n## Recurring wins\n\n- **Making it harder to get into the protected region is the strongest lever.** It worked five times: cold/LIP insertion (0001), a probation FIFO in front of GCLOCK (0004), a probation depth floor (0005), reset-to-0 on promotion (0005–0008), and filtering correlated-reference hits out of promotion credit (0008). Only a real, non-burst hit during probation is a reliable promotion signal.\n- **GCLOCK main + probation FIFO beats the LRU lineage.** Since 0004 it has been the only design that improves both traces without trading one against the other. Its frequency-ranked main needs fewer slots than an LRU protected segment.\n- **Reset-to-0 on promotion is the key safety property.** It keeps main stocked with cold victims, it limited the damage from ghost admission, carried counters and higher ceilings, and it makes cost worries about \"hot main\" mostly theoretical.\n- **Better ranking inside main gave the gains after depth stopped helping.** The steps were: ceiling 3→7 (0006), recency-gated halving decay (0007), linear decay for small main regions (0008) and a veteran grace bit (0008). Each moved cells without regressions, and large-bucket web_0 finally started moving.\n- **The levers depend on scale.** Small buckets (128 slots) reward finer counter tiers and main capacity. Large buckets (1024 slots) reward filtering and history-based grace. Sizing in absolute slots per region beats a single ratio for every size.\n- Cohorts have become safer over time. Cycle 0008 had no real regressions, and its three changes act on paths that don't overlap.\n\n## Recurring failures\n\n- **Fixed-rule insertion in the LRU lineage (0001–0004).** BIP, the floor added on top of BIP, graded promotion and PSEL feedback all traded hm_0 against web_0. The probation and protected regions split fixed bucket space between them, so no single rule suits both traces. That lineage is retired.\n- **Ghost/warm admission.** It failed in two lineages and at two sizings (0003 undersized, 0005 main-sized). It brings back hm_0's near-miss reuse but re-admits web_0's single-return blocks. That is a trace split, not a sizing problem.\n- **Lowering the admission bar or adding counter mass on promotion** (non-zero promotion counters, warm credit). This churns main and decays established blocks faster. The block already gets a full rotation of grace from its position behind the hand.\n- **Probation depth beyond the local optimum.** It was bracketed in 0005–0007 and is closed: floor 64 in small buckets, a quarter in large ones. More depth takes capacity from main, which both traces need.\n- **Admission gates triggered by a hot main** (0007). With reset-to-0 the condition almost never occurs, so these gates did nothing.\n- **Short per-bucket feedback memories** (sample rings, PSEL with symmetric ±1 updates). The state forgot items before they produced a signal, or rewarded the wrong event.\n- **Coarsening counter tiers in small buckets** (bit-length ranking in 0007). It costs small-main capacity.\n\n## Open questions\n\n- Do 0008's three changes stack: small-main linear decay, the veteran grace bit, and the correlated-reference filter? The grace bit and the filter both target counter-0 promotions in large main regions and may overlap.\n- Would widening the correlated-reference window in small buckets (an absolute floor of about 16 slots) bring the web_0 gain to the small size? At what width does it start filtering out hm_0's real reuse?\n- What explains hm_0's remaining large-size gap? Recency, victim ordering, promotion filtering and counter resolution have each moved it only a little. It is most likely long-distance reuse that probation never sees. Could near-miss history be used for something other than admission, for example a probation-eviction tie-break?\n- web_0 at the small size has been the weakest cell in most cycles. Neither depth nor filtering has fully closed it.\n- Per-op-time and metadata-size have never been measured, in any of the eight cycles. This is a harness gap, since neither metric is in the fitness vector. Sweep and scan costs have built up unseen.\n\n## Per-policy notes\n\n- **ClassicLRU / LIP / BIP / DIP-style (0001–0004):** cold insertion helped. Everything layered on top of it traded traces, and the line is retired. The best remaining idea, asymmetric PSEL, was never tried.\n- **CLOCK / GCLOCK without probation (0001–0003):** coarser recency costs no hit-rate. The 2-bit GCLOCK gained in every cell, and the gain grew with bucket size. Ghost admission added nothing.\n- **GCLOCK + probation FIFO (0004–present, main lineage):** the current parent is 0008 v003. Its state is ceiling 7, a recent bit with halving decay, floor-64/quarter probation, reset-to-0, and a correlated-reference promotion filter. Remaining work is in the promotion signal and small-bucket tiers. Main-side levers for hm_0 at the large size look nearly used up.\n- **S3-FIFO-like features:** the probation ring transferred well. Ghost-based re-admission did not transfer to these traces.\n"
}
```

### 15:27:55 chat

I wrote `cross-cycle-insights.md` from all eight cycles in the ledger, using the four required sections in order and no numbers beyond what the ledger itself states. I didn't read the previous version of the file.

- **Recurring wins:** making it harder for items to get into main is the strongest lever. It paid off five times: cold insertion, the probation FIFO, the depth floor, reset-to-0 on promotion, and filtering out burst hits. GCLOCK with a probation FIFO in front has been the leading design since cycle 0004, and reset-to-0 is what keeps it safe. Once probation depth stopped helping, the gains came from better ranking inside main, and each lever only works at one bucket size.
- **Recurring failures:** the LRU/BIP line kept trading hm_0 against web_0. Ghost admission failed in two lineages and at two sizings. Non-zero promotion counters made things worse. Probation depth is used up. Gates that trigger when main is "hot" did nothing, and short feedback memories forgot items before they gave a signal.
- **Open questions:**
  - Do cycle 0008's three changes stack?
  - Would a wider burst-filter window in small buckets help web_0 there?
  - What explains hm_0's remaining gap at the large size?
- **Per-policy notes:** grouped into the LRU family (retired), CLOCK/GCLOCK without probation, the current GCLOCK + probation lineage (parent: cycle 0008 v003), and the S3-FIFO-like features.

Per-op-time and metadata-size have never been measured in any cycle, because the harness doesn't include them in the fitness vector. The file lists this under open questions.


---

- finished: 2026-10-28T15:27:55+00:00
- duration: 28.37s
- status: ok
- input_tokens: 4
- output_tokens: 2487
- cache_read_tokens: 42546
- cache_write_tokens: 14547
- cost_usd: 0.174641
