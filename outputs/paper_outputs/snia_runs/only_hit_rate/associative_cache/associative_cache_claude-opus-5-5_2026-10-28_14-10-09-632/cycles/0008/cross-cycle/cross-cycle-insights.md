# Cross-Cycle Insights (through cycle 0008)

## Recurring wins

- **Making it harder to get into the protected region is the strongest lever.** It worked five times: cold/LIP insertion (0001), a probation FIFO in front of GCLOCK (0004), a probation depth floor (0005), reset-to-0 on promotion (0005–0008), and filtering correlated-reference hits out of promotion credit (0008). Only a real, non-burst hit during probation is a reliable promotion signal.
- **GCLOCK main + probation FIFO beats the LRU lineage.** Since 0004 it has been the only design that improves both traces without trading one against the other. Its frequency-ranked main needs fewer slots than an LRU protected segment.
- **Reset-to-0 on promotion is the key safety property.** It keeps main stocked with cold victims, it limited the damage from ghost admission, carried counters and higher ceilings, and it makes cost worries about "hot main" mostly theoretical.
- **Better ranking inside main gave the gains after depth stopped helping.** The steps were: ceiling 3→7 (0006), recency-gated halving decay (0007), linear decay for small main regions (0008) and a veteran grace bit (0008). Each moved cells without regressions, and large-bucket web_0 finally started moving.
- **The levers depend on scale.** Small buckets (128 slots) reward finer counter tiers and main capacity. Large buckets (1024 slots) reward filtering and history-based grace. Sizing in absolute slots per region beats a single ratio for every size.
- Cohorts have become safer over time. Cycle 0008 had no real regressions, and its three changes act on paths that don't overlap.

## Recurring failures

- **Fixed-rule insertion in the LRU lineage (0001–0004).** BIP, the floor added on top of BIP, graded promotion and PSEL feedback all traded hm_0 against web_0. The probation and protected regions split fixed bucket space between them, so no single rule suits both traces. That lineage is retired.
- **Ghost/warm admission.** It failed in two lineages and at two sizings (0003 undersized, 0005 main-sized). It brings back hm_0's near-miss reuse but re-admits web_0's single-return blocks. That is a trace split, not a sizing problem.
- **Lowering the admission bar or adding counter mass on promotion** (non-zero promotion counters, warm credit). This churns main and decays established blocks faster. The block already gets a full rotation of grace from its position behind the hand.
- **Probation depth beyond the local optimum.** It was bracketed in 0005–0007 and is closed: floor 64 in small buckets, a quarter in large ones. More depth takes capacity from main, which both traces need.
- **Admission gates triggered by a hot main** (0007). With reset-to-0 the condition almost never occurs, so these gates did nothing.
- **Short per-bucket feedback memories** (sample rings, PSEL with symmetric ±1 updates). The state forgot items before they produced a signal, or rewarded the wrong event.
- **Coarsening counter tiers in small buckets** (bit-length ranking in 0007). It costs small-main capacity.

## Open questions

- Do 0008's three changes stack: small-main linear decay, the veteran grace bit, and the correlated-reference filter? The grace bit and the filter both target counter-0 promotions in large main regions and may overlap.
- Would widening the correlated-reference window in small buckets (an absolute floor of about 16 slots) bring the web_0 gain to the small size? At what width does it start filtering out hm_0's real reuse?
- What explains hm_0's remaining large-size gap? Recency, victim ordering, promotion filtering and counter resolution have each moved it only a little. It is most likely long-distance reuse that probation never sees. Could near-miss history be used for something other than admission, for example a probation-eviction tie-break?
- web_0 at the small size has been the weakest cell in most cycles. Neither depth nor filtering has fully closed it.
- Per-op-time and metadata-size have never been measured, in any of the eight cycles. This is a harness gap, since neither metric is in the fitness vector. Sweep and scan costs have built up unseen.

## Per-policy notes

- **ClassicLRU / LIP / BIP / DIP-style (0001–0004):** cold insertion helped. Everything layered on top of it traded traces, and the line is retired. The best remaining idea, asymmetric PSEL, was never tried.
- **CLOCK / GCLOCK without probation (0001–0003):** coarser recency costs no hit-rate. The 2-bit GCLOCK gained in every cell, and the gain grew with bucket size. Ghost admission added nothing.
- **GCLOCK + probation FIFO (0004–present, main lineage):** the current parent is 0008 v003. Its state is ceiling 7, a recent bit with halving decay, floor-64/quarter probation, reset-to-0, and a correlated-reference promotion filter. Remaining work is in the promotion signal and small-bucket tiers. Main-side levers for hm_0 at the large size look nearly used up.
- **S3-FIFO-like features:** the probation ring transferred well. Ghost-based re-admission did not transfer to these traces.
