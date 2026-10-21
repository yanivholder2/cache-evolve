# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:28:00+00:00

## timeline

### 04:28:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hit-rate is decided by who gets into the scarce normal tier and who survives in the stash, not by where items sit or which free slot they use. Each cycle that produced a winner did it through admission or retention control.\n- 2Q/SLRU-like split: fresh keys face a high admission bar (a stash-hit threshold, in the lineage since 0001), while proven ex-residents re-enter cheaply via pre-credit and are protected by MRU stash placement (0002). Every real gain so far came from building up this split.\n- The stash being larger than the normal tier is what makes it work. It can hold unproven keys and demoted ex-residents cheaply, so filtering and recovery at the stash edge cost little hit-rate.\n\n## Recurring failures\n\n- Placement-only changes (MRU landing in a bucket in 0001, spilling into a free probing slot in 0001) either lose or rarely trigger. Normal buckets are almost always full together, and position is coupled to the probing-promotion bar.\n- Gating the target→probing edge did nothing (0002). Front landing and pre-credit already filter it.\n- Tightening the stash threshold gave diminishing returns: a small gain at 3 in 0002, and nothing on top of MRU re-entry in 0003. Don't go beyond it.\n- Once MRU stash re-entry is in place, the second-order levers all came out flat (0003): stash victim choice (CLOCK-like scan), cascade routing, and stacked admission. They only reshuffle victims the stash can recover cheaply. Stacking retention mechanisms that protect the same items adds nothing.\n- Cross-bucket cascade damage looked important in 0001. It stopped mattering for hit-rate once the stash recovered demoted items.\n\n## Open questions\n\n- Per-op-time and metadata-size have never been measured for any winner, after three cycles. This is the biggest gap. Cascade-free routing (0003 v003) is the leading per-op-time candidate because it cost no hit-rate.\n- Generality: every result is from one LIRS trace at one cache size. Is the plateau a property of the design or of the trace? What happens to MRU re-entry under scans or a larger working set?\n- Build integrity: a variant scored identically to its parent (0003 v001). Verify that overlays actually reach the evaluated build before treating nulls as real.\n- Structural capacity, such as the stash-to-normal slot ratio or stash size, is untested and is the remaining plausible hit-rate lever.\n\n## Per-policy notes\n\n- Stash/normal-tier tiered design (the only policy family in the ledger so far): the current lineage is 0001 v003 (two-hit stash admission) → 0002 v003 (MRU stash re-entry for demoted items with pre-credit) → 0003 (flat; v002's CLOCK-like stash scan is the nominal winner at noise level). On this trace it looks saturated for hit-rate.\n- Within this policy, the stash→target edge is the lever with leverage. The target→probing edge and placement within a bucket are low-yield.\n- No other eviction policies or traces have been evaluated yet, so there are no comparative per-policy observations.\n"
}
```

### 04:28:21 thinking



### 04:28:21 chat

I wrote `cross-cycle-insights.md` with the four required sections, built only from the three cycles in the ledger. Overall, the design looks saturated for hit-rate on the one trace tested so far.

- **Recurring wins:** every gain came from controlling who gets into the normal tier and who survives in the stash. New keys need two stash hits to get in (cycle 0001). Items pushed out of the normal tier go back into the stash at the most-recently-used end with a head start on that count (cycle 0002). Changing where items are placed never produced a gain.
- **Recurring failures:**
  - **Placement-only changes:** these lost hit-rate or rarely took effect.
  - **Target→probing edge:** adding a hit-count bar there did nothing.
  - **Stash threshold:** raising it beyond 2 gave little and is not worth pushing further.
  - **Cycle 0003 levers:** all three (a higher threshold, a different choice of which stash item to evict, rerouting evictions) were flat once the cycle-0002 retention change was in place.
- **Open questions:**
  - **Speed and memory:** no winner has been measured for per-op-time or metadata-size yet. That is the biggest gap.
  - **Generality:** every result comes from one LIRS trace at one cache size.
  - **Build check:** one cycle-0003 variant scored exactly the same as its parent, so it should be checked that its change actually reached the build.
  - **Capacity:** the ratio of stash slots to normal-tier slots has not been tried.
- **Per-policy notes:** the ledger has only one policy family, the stash plus normal-tier design. The notes trace its line of winners and say that no other policies have been compared yet.

Files are in /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0003/cross-cycle:
- cross-cycle-insights.md


---

- finished: 2026-10-21T04:28:21+00:00
- duration: 21.36s
- status: ok
- input_tokens: 4
- output_tokens: 1817
- cache_read_tokens: 35444
- cache_write_tokens: 7951
- cost_usd: 0.107446
