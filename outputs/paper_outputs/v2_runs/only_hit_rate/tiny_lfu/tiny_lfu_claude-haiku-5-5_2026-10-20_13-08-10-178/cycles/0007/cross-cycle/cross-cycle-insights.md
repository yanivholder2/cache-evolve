# Cross-cycle insights

## Recurring wins

- Reducing collision over-counting in the frequency sketch is the lever that has most consistently produced hit-rate gains: sketch widening (0004), conservative update (0005), and an extra hash row (0007) all acted on the same estimator error. Gains appear to overlap rather than stack independently.
- Keeping the sketch's min-across-rows estimate well supplied with independent rows matters; removing depth hurt (0005) and adding it helped (0007).
- The probation/protected split inside the main region is a real lever (0003, 0006), but its effect is non-monotone: the incumbent ratio sits near the best setting, and pushing the protected share higher has reversed a gain (0007).
- Promoting entries on every hit has held up better than gating promotion on sketch estimates (0007).

## Recurring failures

- Decaying or aging sketch history on a fixed schedule or access-count trigger has consistently lost hit-rate (0001, 0005): admission needs accumulated frequency at these capacities.
- Changing when or whether admission sees a key has lost hit-rate: a recency window ahead of the LRU (0002), a doorkeeper that withholds first-touch keys from the sketch (0006), and sketch-gated promotion (0007). Frequency counts lag recency, and correction at estimate time does not replace withheld increments.
- Rewrites of key-to-counter mapping and sketch-side changes that shift estimates the wrong way did not pay off (0001).
- Layout-only changes (narrower counters, flattened or hoisted indices) preserve hit-rate by construction, but their claimed gains (footprint, per-op cost) sit on dimensions the fitness vector does not measure (0001, 0002, 0003). They have advanced only by not regressing, and should not be stacked until a metadata or timing signal exists.
- Halved protected caps and push-to-front demotion lost hit-rate against the incumbent (0004).

## Open questions

- Winner attribution is unresolved: cycle 0007 lists v001 as a Winner but the only reported gain is v002's depth change. Confirm which variant actually advanced before building on either.
- Do the estimator gains (width, conservative update, depth) hold on other workloads and cache sizes, and do they overlap or stack?
- Would a memory or timing dimension in the fitness vector confirm the layout gains and price the counter, row, and per-op costs of the estimator changes?
- Where does adding sketch rows or widening stop paying for its memory, and where does the protected-share optimum sit on other traces?
- Is there an aging trigger, not tied to access count, that sheds stale frequency without weakening admission?

## Per-policy notes

- TinyLFU admission and frequency sketch: estimator error (width, conservative update, depth) is the productive direction. Decay, aging, and first-touch gating have all lost hit-rate. Keep the sketch's history intact.
- Segmented LRU (main region): the probation/protected split helps; keep the protected share near the incumbent ratio, and keep demotion placement at the back of probation.
- Promotion rule: promote on every hit; do not gate promotion on sketch estimates.
- Window / LRU front region: a recency window ahead of the main LRU with admission at window exit lost hit-rate on this trace; drop this structure at current capacity.
- All observations come from one trace at one cache size and should be treated as provisional.
