# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:24:44+00:00

## timeline

### 09:24:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Protecting keys that have proven they get reused pays off on workload-a. Every Winner so far (shrinking the small queue twice, raising the main frequency cap) gave more room or more retention to keys with demonstrated repeat reuse.\n- Shrinking the small queue (to 10%, then to 5%) gave consistent but shrinking hit-rate gains. The gain comes from a larger main queue, not from a larger ghost.\n- Main-queue retention (the frequency cap) is a newer axis. It gave a modest gain, and it can be combined with the partition changes.\n- Single-change variants with a clear lineage gave attributable results. Variants that bundled two changes (cycle 0001 v002) could not be attributed.\n\n## Recurring failures\n\n- Making re-admission or graduation more generous keeps losing hit-rate. Examples: a lower graduation threshold, a larger or decoupled ghost, a freq credit for ghost re-admits. On this workload, one extra reuse signal is weak evidence, and the main capacity it takes comes out of keys with proven reuse.\n- Making re-admission much stricter by inserting at the main tail also lost. Front-of-main, freq-0 re-admission sits between those two and has held up.\n- Per-op-time and metadata-size have gone unmeasured for three cycles in a row, even though every Continuation asked for them. The incumbent lineage keeps lengthening its linear scans (a larger ghost, a higher cap), so the unmeasured cost keeps growing.\n- Carried-forward actions get dropped. The indexed-structure port took until cycle 3, and combining it with the policy Winner has still not been done.\n\n## Open questions\n\n- How much per-op-time does the behaviour-exact indexed port save, and how much metadata-size does it add? Does it stack cleanly with the current policy baseline (5% split, cap of 7)?\n- How long do eviction sweeps get under the higher frequency cap? Does raising the cap above 7, or setting the graduation threshold separately from the cap, add hit-rate?\n- Can a re-admission probation window counted in accesses, somewhere between tail and front insertion, beat front insertion? Would that make a larger ghost useful?\n- Do the partition and cap gains hold on other traces and cache sizes, or are they specific to workload-a?\n- Was cycle 0001 v002's regression caused by the lower threshold or by the frequency reset? This is low priority.\n\n## Per-policy notes\n\n- S3-FIFO: the current baseline is a 5% small queue, a main frequency cap of 7, a ghost sized with main, and front-of-main freq-0 ghost re-admission. The partition curve looks close to flat below 10%, so further shrinking is likely to give diminishing returns. The ghost now does most of the filtering.\n- S3-FIFO: the indexed slab/index structure is verified as behaviour-neutral and is ready to adopt. Its cost and benefit are unmeasured.\n"
}
```

### 09:25:03 thinking



### 09:25:03 chat

I wrote `cross-cycle-insights.md` from the three-cycle ledger, using the four required sections.

- **Recurring wins:** On workload-a, changes that protect keys with proven reuse keep paying off. Shrinking the small queue (to 10%, then to 5%) gave gains that are getting smaller, and raising the main frequency cap gave a modest gain. Changing one thing per variant is what made results attributable.
- **Recurring failures:** Two kinds of change kept losing hit-rate. Making re-admission or graduation more generous lost: a lower graduation threshold, a larger ghost queue, or a frequency credit for re-admitted keys. Making it much stricter by inserting at the main tail also lost. Per-op-time and metadata-size have never been measured, and the indexed structure has never been combined with the winning policy, even though every cycle's Continuation asked for both.
- **Open questions:** What the indexed port actually costs and saves, and whether it stacks with the current policy. How long eviction sweeps get under the higher cap, and whether a cap above 7 helps. Whether a re-admission probation window counted in accesses beats front insertion. Whether the gains hold on other traces and cache sizes.
- **Per-policy notes (S3-FIFO):** The current baseline is a 5% small queue, a frequency cap of 7, and re-admission at the front of main with zero frequency. Shrinking the partition further looks close to flat. The indexed structure gives identical results and is ready to adopt, but its costs haven't been measured.


---

- finished: 2026-10-21T09:25:03+00:00
- duration: 18.66s
- status: ok
- input_tokens: 4
- output_tokens: 1643
- cache_read_tokens: 35314
- cache_write_tokens: 7764
- cost_usd: 0.102051
