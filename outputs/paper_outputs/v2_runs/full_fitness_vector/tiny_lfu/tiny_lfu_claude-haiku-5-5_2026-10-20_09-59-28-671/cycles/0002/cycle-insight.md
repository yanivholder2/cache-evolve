# Cycle 0002 Insight

## Cohort outcome

The cohort tried three kinds of change: hashing each key once and reusing it across the admission check and sketch update (v001), packing the protected flag into an existing node field while cutting resident-list anchors (v002), and a one-hit doorkeeper in front of the frequency sketch (v003). All three Winners kept hit rate close to their parent, and the gains were mostly in per-op time or metadata size, not in hit rate.

## Patterns

- Constant-factor work was the main source of gains. Hoisting the hash out of `put()` produced a clear put-path speedup, but it also moved hit rate slightly, so it is not behaviour-preserving as written.
- Metadata reduction and hit rate decoupled. Packing the protected state into existing fields shrank metadata without changing hit rate, so the bookkeeping cost of segmentation can be trimmed independently of the policy.
- The doorkeeper barely moved hit rate on this workload. One-hit-wonder pollution of the sketch does not appear to limit the admission gate here, and the change mostly added handle state and per-put work.
- Several gains and regressions are unverified attributions. Speedups were attributed to shared helpers and regressions to specific accessors or the doorkeeper without inspecting the source, and all evidence comes from one trace.

## Open questions

- Do the precomputed-hash helpers produce the same counter indices as the public sketch methods on the same keys? This determines whether v001's hit-rate drop is a bug or a real trade-off.
- Is the doorkeeper's memory cost fully reported? Its storage does not appear in the metadata breakdown.
- Does any of this hold on a trace where one-hit keys dominate, or on a second trace at all?

## Continuation

Pursue the hash-reuse direction from v001, but first confirm equivalence of the hashed helper indices so the hit-rate drop is resolved. Keep the metadata-packing approach from v002, since it cut cost without moving hit rate. Drop the doorkeeper (v003) as a direction unless a trace where one-hit keys dominate shows a real hit-rate effect.
