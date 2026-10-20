# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:42:16+00:00

## timeline

### 14:42:26 thinking



### 14:42:31 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Stash-hit landing at the MRU end of the target bucket is the one placement change that has held up. It was the only winner in cycle 1 and remained the base in later cycles.\n- Admission filtering on reuse helps a little. Requiring a second hit before stash promotion (cycle 2) and, in weaker form, a saturating reuse counter (cycle 6) both point the same way: one-off hits displace residents that have not yet proven reuse.\n- Stacking the second-hit gate on MRU landing (cycle 3) gave a small further gain. The effect is small and rests on one workload and one capacity.\n\n## Recurring failures\n\n- Moving items out of their current tier hurts. Removing and re-inserting probing-tier hits (cycle 1) and skipping the target retry for foreign evictions (cycle 1) both lowered hit-rate. The unconditional direct-to-stash shortcut (cycle 2) recovered only part of its loss when gated on free stash capacity, so the removed retry was valued under pressure.\n- Using stash-marker state as an eviction signal hurts. Preferring never-re-hit stash entries as victims (cycle 3) discarded recently arrived items before they could prove reuse.\n- Early readmission hurts slightly. The ghost-history fast path (cycle 5) let recently evicted keys skip the stash wait and lost a little hit-rate.\n- Small refinements of landing, gating, and victim order show no measurable effect. Occupancy-conditional landing (cycle 2), occupancy-conditional gating (cycle 4), earliest-landed victim order (cycle 4), LRU-end demotion placement (cycle 5), and the gate reorder (cycle 6) all matched their parent. Occupancy counters (cycles 5 and 6) also matched parent, so their speed benefit remains unmeasured.\n- Spare-bit marker relocation (cycle 3, cycle 4) left hit-rate unchanged. This does not show that any metadata saving was realised.\n\n## Open questions\n\n- Does any winning lever (MRU landing, second-hit gate, reuse counter) hold on workloads and capacities other than the single LIRS trace? Every cycle so far used that one trace.\n- Per-op-time and metadata-size have not been measured in any cycle. The cost side of every lever, including the reuse counter's extra bits per entry, is unknown.\n- Several insights rest on hypothesis text rather than the overlay diff (cycles 2, 4, 5, 6). Whether the implementations match their descriptions, including the spare-bit layout and the reuse counter's reset on demotion and eviction, is unverified.\n- Winner labelling is inconsistent. Some winners were inherited or carried forward with no change from parent (cycles 2 and 4), and one was labelled a winner despite a slight hit-rate drop (cycle 5). The baseline used for \"win\" needs to be fixed.\n\n## Per-policy notes\n\n- All six cycles explore one policy lineage: the probing, stash, and normal tier design evaluated on the LIRS trace. No cross-policy comparison exists yet.\n- Stash tier: landing position and reuse-based promotion are the only levers with a hit-rate signal. Victim selection and occupancy-dependent variants have shown no effect.\n- Probing tier: residency is valuable. Changes that remove or re-insert probing hits have consistently lost ground.\n- Normal/target tier: foreign-eviction retries and demotion placement carry real but conditional value. They are not promising levers by themselves.\n- Ghost history: readmission shortcuts have only a negative measured result so far.\n"
}
```

### 14:42:34 chat

I wrote `cross-cycle-insights.md` in the output directory (`cycles/0006/cross-cycle/`), using the four required sections in order. I built it only from the ledger digest in the prompt and did not read any previous version of the file.

The main signals across cycles 0001–0006:

- **Recurring wins:** MRU landing for stash hits has held up since cycle 1. Requiring reuse before promotion, via a second-hit gate or a saturating counter, has shown small hit-rate gains.
- **Recurring failures:** Moving items out of their tier (probing re-insertion, skipping the foreign-eviction retry) hurt hit-rate. Using the stash marker as an eviction signal hurt, and the ghost-history readmission path lost slightly. Many small landing, gate, and victim-order refinements matched their parent exactly.
- **Open questions:** Every cycle used one LIRS trace at one capacity, so generalisation is untested. Per-op-time and metadata-size were never measured. Several insights rest on hypothesis text rather than the overlay diff. Winner labelling is inconsistent: some winners matched their parent, and one was labelled a winner despite a slight drop.
- **Per-policy notes:** All six cycles explore a single policy lineage (probing, stash, and normal tiers), so there is no cross-policy comparison yet.


---

- finished: 2026-10-20T14:42:34+00:00
- duration: 18.49s
- status: ok
- input_tokens: 4
- output_tokens: 3503
- cache_read_tokens: 37945
- cache_write_tokens: 11544
- cost_usd: 0.00444
