# Cross-cycle insights

## Recurring wins

- Per-op-time gains that leave eviction order and hit rate unchanged keep recurring. Hoisting the hash, inlining get-path accessors, and splitting the put-side unlink/promotion path all improved timing without moving hit rate.
- Keeping per-row sketch seeds in a module-level `const` table, not per instance, removed the struct growth that per-instance caching caused and kept the speedup.
- Packing protected state into existing node fields cut metadata without changing hit rate, so segmentation's bookkeeping cost can be trimmed independently of the policy.
- Collision spread is the lever that has moved hit rate. Conservative counter updates and reshaping the sketch toward more columns and fewer rows both reduced inflation from colliding keys and gave small gains.

## Recurring failures

- Recency rotation of rejected admission victims lost on both hit rate and per-op time. The frequency gate already does that protective work.
- Admission and aging tweaks have not raised hit rate on this workload: a one-hit-wonder doorkeeper, a recency window in front of the gate, halved sketch aging, and admission tie-breaking all left hit rate flat or lower. Stale counts and one-hit pollution do not appear to be the bottleneck.
- Capping or shrinking the protected segment cost hit rate and per-op time. Segmentation's gain seems to depend on the protected set turning over, not on bounding it.
- Layouts that change which keys collide cost hit rate, and layouts that restore collision fidelity cost speed. Cache-line grouping gave a clear speedup but changed collisions; restoring public per-row indexing recovered hit rate but reintroduced per-row work. No variant has yet had both.
- Changes that add a per-row lookup or extra pass on every access consistently regressed get and put timing.

## Open questions

- Does the collision-spread gain from fewer rows and more columns hold on a second trace and at other cache sizes?
- Can conservative counter updates keep their collision benefit without an extra pass over all rows?
- Can public-equivalent per-row indices be produced with less work than the seeded mixing, so index fidelity and speed stop trading off?
- Do the `const`-table seed indices match the per-access derivation for every key, or only in aggregate hit rate?
- Do put and get timing shifts survive repeat runs? Many were single-run results.
- Does a protected-capacity setting exist that exercises demotion without losing hit rate?
- Does any of this hold on a trace where one-hit keys dominate, or on a second trace at all?
- What caused the unexplained metadata and struct size movements in several variants, given that the hypotheses forbade them?

## Per-policy notes

- Admission gate (frequency sketch plus frequency-based victim check): the main source of protection. Changes to the gate's inputs, such as doorkeepers, windows, aging, and tie-breaks, have not improved hit rate. Changes to the sketch's collision behaviour are the exception that has moved it.
- Eviction and recency (LRU-style list ordering): reordering rejected victims was harmful. Pointer-work trims that preserve eviction order are safe, and those are the most reliable speed gains.
- Segmentation (probation/protected): segment size is sensitive. Capping or shrinking the protected share hurt, and tail demotion has seen little hit-rate effect. Metadata cost is the main drawback, and it can be reduced by packing flags into existing fields.
- Frequency sketch layout: seeds, row/column shape, and block grouping drive both hit rate and speed. Layout changes that alter collisions trade hit rate for speed, and the best-performing direction so far is the collision-spread change.
- Evidence base: all results are from one trace (workload-a) at one cache size, mostly from single runs, and overlay source was not inspected. Treat every attribution as provisional.
