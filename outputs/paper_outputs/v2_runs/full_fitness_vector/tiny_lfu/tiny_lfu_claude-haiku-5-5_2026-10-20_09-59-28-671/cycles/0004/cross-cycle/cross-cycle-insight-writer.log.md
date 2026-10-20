# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:20:27+00:00

## timeline

### 10:20:34 thinking



### 10:20:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Constant-factor work on the admission path has been the most reliable source of gains: relink guards, hoisted hashing, and cached sketch seeds all cut per-op cost.\n- Changes that keep eviction order and counter indices intact hold hit rate flat while reducing cost.\n- Metadata reduction can be decoupled from policy: packing state into existing fields shrank bookkeeping without moving hit rate.\n\n## Recurring failures\n\n- Changes that alter admission or aging have not raised hit rate: recency rotation of rejected victims, a one-hit doorkeeper, shorter sketch aging, and a small recency window all left it flat or lower. The frequency gate appears to already do the protective work.\n- Capping the protected segment cost hit rate. The segmentation gain seems to depend on the protected set turning over.\n- Switching index schemes trades fidelity against speed: matching the public per-row indexing restored hit rate but gave back most of the speedup.\n- Bookkeeping keeps growing per node, per instance, or in list work, often without being reflected in reported metadata.\n- Timing attributions are routinely unverified: speedups and regressions were assigned to specific helpers or accessors without source inspection.\n\n## Open questions\n\n- Does the segmentation hit-rate gain hold on any trace other than the one used so far?\n- Do the cached and hoisted seed/hash helpers produce exactly the same counter indices as the public path for every key? Parity so far is consistent with yes but not proven.\n- Can the seed table be placed in a `const` item so the speedup survives without per-instance growth?\n- Are the unexplained timing shifts (put-side changes, latency drops in segmentation) real under repeat runs?\n- Is the doorkeeper's storage fully accounted for in reported metadata, and does the doorkeeper conclusion change on a trace where one-hit keys dominate?\n\n## Per-policy notes\n\n- TinyLFU frequency admission gate: the strongest lever so far is per-op cost on the sketch path. Hit rate has resisted changes to the gate itself, including aging period, one-hit filtering, and recency windows.\n- Sketch indexing and seeds: index-scheme changes are the main fidelity-versus-speed trade-off. Cached seeds look hit-rate neutral and should be verified against the public indexing before further reuse.\n- Segmented (probation/protected) main region: the only source of hit-rate movement observed, but small, and it adds per-node and per-cache metadata. Capping it hurt. Needs a second trace before it is built on.\n- Window and recency rotation: no hit-rate benefit observed; both added list work. Drop as directions on the current workload.\n- Evidence base: every result across all cycles comes from one trace (workload-a), so no effect has been shown to generalize.\n"
}
```

### 10:20:41 chat

I wrote `cross-cycle-insights.md` in the cross-cycle output dir, using the four required sections in order. I built it only from the ledger digest in the prompt and did not open the previous version of the file.

The main points:

- **Recurring wins:** constant-factor work on the admission path (relink guards, hoisted hashing, cached seeds) and changes that keep eviction order and counter indices intact. Metadata packing also cut cost without moving hit rate.
- **Recurring failures:** changes to admission or aging (recency rotation, doorkeeper, shorter aging, recency window) did not raise hit rate. Capping the protected segment lost hit rate. Matching the public index scheme restored hit rate but gave back most of the speedup.
- **Open questions:** whether the segmentation gain holds on another trace, whether the cached seeds match the public path's counter indices for every key, whether the timing shifts survive repeat runs, and whether doorkeeper storage is fully reported.
- **Per-policy notes:** grouped under the frequency gate, sketch indexing, the segmented main region, and the window/recency work.

Every result so far comes from one trace (workload-a), and the file says so. No numbers are included, since the digest gives none.


---

- finished: 2026-10-20T10:20:42+00:00
- duration: 15.29s
- status: ok
- input_tokens: 4
- output_tokens: 2675
- cache_read_tokens: 36480
- cache_write_tokens: 9298
- cost_usd: 0.003562
