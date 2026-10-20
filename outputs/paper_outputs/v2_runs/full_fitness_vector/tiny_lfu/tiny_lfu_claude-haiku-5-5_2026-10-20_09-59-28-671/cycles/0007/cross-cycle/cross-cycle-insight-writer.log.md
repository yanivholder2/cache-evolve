# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:32:21+00:00

## timeline

### 10:32:31 thinking



### 10:32:37 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Per-op speedups that leave eviction order and counter indices untouched have repeatedly kept hit rate at parent values (hoisting the hash, packing the protected flag into existing fields, moving sketch seeds into a shared constant table).\n- Moving per-instance state out of the cache struct removes metadata growth. Shared constant tables have been the cleanest way to keep the seed speedup without struct growth.\n- Gets and puts respond differently. Shared-path changes (unlink/promotion split, get-path accessor inlining) have shown put-side or get-side timing shifts, but each rests on single runs.\n\n## Recurring failures\n\n- Admission-side changes that alter what the frequency sketch sees have not raised hit rate: the one-hit doorkeeper, shorter frequency aging, the window in front of the gate, and admission tie-breaking were all flat or negative. Stale counts and one-hit pollution do not appear to be the limiting factor on this workload.\n- Bounding or shrinking the protected segment loses hit rate. Capping it without demotion, and halving its share, both cost hit rate and per-op time.\n- Recency rotation of rejected admission victims hurt both hit rate and per-op time.\n- Restoring public per-row sketch indexing costs per-op time on both get and put. No formulation so far keeps both the index fidelity and the speed gain.\n- Layouts that change which keys collide in the sketch buy speed at a hit-rate cost. Taking the column from a per-row seed removed the speed benefit without restoring collision spread.\n- Single-run timing has repeatedly failed to hold up as a signal. Several \"Winners\" were labelled on timing shifts that were not separated from noise, and a Winner label has not implied a net gain.\n\n## Open questions\n\n- Cross-trace validation is the largest gap. Every result so far comes from one trace (workload-a, size 2048 in the latest cycle). Does any hit-rate effect hold on a second trace, especially one where one-hit keys dominate?\n- Can per-row index fidelity be produced with less per-row work than the current mixing, so correctness does not cost the speed gain?\n- Do precomputed-hash and const-table helpers produce the same counter indices as the public path for every key, or only matching aggregate hit rate?\n- Can cache-line or block grouping keep its get-path gain while preserving collision-spread behaviour?\n- Do the put-side and get-side timing shifts survive repeat runs?\n- Is there a protected capacity that still exercises the demotion path without losing hit rate?\n- Are the unexplained size and struct movements real, and what caused them? The doorkeeper and window storage do not appear in reported metadata, so reported sizes may undercount.\n\n## Per-policy notes\n\n- TinyLFU admission gate (frequency sketch): the most sensitive area for hit rate, through indexing collisions. Tie-breaking, doorkeeper, and aging changes did not help. Index-altering changes moved hit rate; index-preserving speedups did not.\n- Protected/probation segments (SLRU-style main cache): protected capacity is the live lever. Demotion never moved hit rate at full capacity, and reducing capacity cost hit rate. Protected-tail turnover remains untested as a source of gain.\n- Victim selection and eviction order: recency rotation of rejected victims is a recurring failure. Changes that keep eviction order fixed have been the safest source of speed gains.\n- Window (W-TinyLFU style): a recency window in front of the gate did not improve hit rate and added list work on both get and put. Not a current direction on this workload.\n- Metadata and storage: constant tables for sketch seeds keep the speed without struct growth. Doorkeeper and window bookkeeping are not reflected in reported metadata.\n"
}
```

### 10:32:39 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory for cycle 0007. It has the four required sections in order, built only from the inlined ledger digest. I did not read the previous insights file.

The main points:

- **Recurring wins:** speedups that keep eviction order and counter indices unchanged have kept hit rate flat. Moving seeds into a shared constant table removed struct growth.
- **Recurring failures:** admission-side changes (doorkeeper, shorter aging, window, tie-breaking) did not raise hit rate. Shrinking or capping the protected segment lost hit rate. Restoring public per-row indexing costs per-op time.
- **Open questions:** every result comes from one trace (workload-a), so cross-trace validation is the largest gap. Repeat runs are also needed for the timing signals.
- **Per-policy notes:** grouped under the TinyLFU admission gate, protected/probation segments, victim selection, the window, and metadata.


---

- finished: 2026-10-20T10:32:39+00:00
- duration: 18.27s
- status: ok
- input_tokens: 4
- output_tokens: 3368
- cache_read_tokens: 39008
- cache_write_tokens: 12599
- cost_usd: 0.004594
