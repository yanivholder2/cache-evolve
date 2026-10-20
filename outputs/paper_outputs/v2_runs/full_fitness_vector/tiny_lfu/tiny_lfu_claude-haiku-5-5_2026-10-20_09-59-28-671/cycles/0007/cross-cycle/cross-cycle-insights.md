# Cross-cycle insights

## Recurring wins

- Per-op speedups that leave eviction order and counter indices untouched have repeatedly kept hit rate at parent values (hoisting the hash, packing the protected flag into existing fields, moving sketch seeds into a shared constant table).
- Moving per-instance state out of the cache struct removes metadata growth. Shared constant tables have been the cleanest way to keep the seed speedup without struct growth.
- Gets and puts respond differently. Shared-path changes (unlink/promotion split, get-path accessor inlining) have shown put-side or get-side timing shifts, but each rests on single runs.

## Recurring failures

- Admission-side changes that alter what the frequency sketch sees have not raised hit rate: the one-hit doorkeeper, shorter frequency aging, the window in front of the gate, and admission tie-breaking were all flat or negative. Stale counts and one-hit pollution do not appear to be the limiting factor on this workload.
- Bounding or shrinking the protected segment loses hit rate. Capping it without demotion, and halving its share, both cost hit rate and per-op time.
- Recency rotation of rejected admission victims hurt both hit rate and per-op time.
- Restoring public per-row sketch indexing costs per-op time on both get and put. No formulation so far keeps both the index fidelity and the speed gain.
- Layouts that change which keys collide in the sketch buy speed at a hit-rate cost. Taking the column from a per-row seed removed the speed benefit without restoring collision spread.
- Single-run timing has repeatedly failed to hold up as a signal. Several "Winners" were labelled on timing shifts that were not separated from noise, and a Winner label has not implied a net gain.

## Open questions

- Cross-trace validation is the largest gap. Every result so far comes from one trace (workload-a, size 2048 in the latest cycle). Does any hit-rate effect hold on a second trace, especially one where one-hit keys dominate?
- Can per-row index fidelity be produced with less per-row work than the current mixing, so correctness does not cost the speed gain?
- Do precomputed-hash and const-table helpers produce the same counter indices as the public path for every key, or only matching aggregate hit rate?
- Can cache-line or block grouping keep its get-path gain while preserving collision-spread behaviour?
- Do the put-side and get-side timing shifts survive repeat runs?
- Is there a protected capacity that still exercises the demotion path without losing hit rate?
- Are the unexplained size and struct movements real, and what caused them? The doorkeeper and window storage do not appear in reported metadata, so reported sizes may undercount.

## Per-policy notes

- TinyLFU admission gate (frequency sketch): the most sensitive area for hit rate, through indexing collisions. Tie-breaking, doorkeeper, and aging changes did not help. Index-altering changes moved hit rate; index-preserving speedups did not.
- Protected/probation segments (SLRU-style main cache): protected capacity is the live lever. Demotion never moved hit rate at full capacity, and reducing capacity cost hit rate. Protected-tail turnover remains untested as a source of gain.
- Victim selection and eviction order: recency rotation of rejected victims is a recurring failure. Changes that keep eviction order fixed have been the safest source of speed gains.
- Window (W-TinyLFU style): a recency window in front of the gate did not improve hit rate and added list work on both get and put. Not a current direction on this workload.
- Metadata and storage: constant tables for sketch seeds keep the speed without struct growth. Doorkeeper and window bookkeeping are not reflected in reported metadata.
