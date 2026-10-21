# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:23:24+00:00

## timeline

### 04:23:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that decide *what is admitted or promoted* paid off; changes that decide *where things sit* mostly did not.\n- Two wins so far: MRU placement on hit-promotion (cycle 1) and an unconditional second-sighting admission filter (cycle 3). Both are simple, low-machinery changes.\n- Both wins rest on one idea: protect recently useful entries and keep one-hit-wonders out.\n\n## Recurring failures\n\n- Placement-position tweaks (insert, demotion, eligibility) added nothing beyond the cycle 1 win. Several reproduced the parent's hit-rate exactly.\n- Adding movement (bidirectional swaps, more promotions, wider probe neighbourhoods) caused cascades or let cold entries crowd out hot ones.\n- Refining the admission filter (counters with aging, victim-conditional exemption, doorkeeper bit filters) lost hit-rate every time. Smaller or faster-resetting state raised collisions and forgot once-seen keys. \"Victim never hit\" is a weak value signal.\n- Exact-tie results recur (cycles 2 and 3). The overlay may not be on the executed path, and authors cannot read overlay sources, so these null results are ambiguous.\n- \"Winner\" labels can mean \"best of a weak cohort\" or \"matched the parent\", not a real improvement (cycles 2 and 4).\n\n## Open questions\n\n- Do the wins generalize beyond workload-a and the current cache size? Every conclusion comes from a single trace.\n- What do the wins cost? Hit-rate is the only dimension ever reported. Per-op-time and metadata-size are unmeasured, which matters for the filter's per-segment state and miss-path hashing.\n- Are the exact-tie variants live code? Is victim selection still untested because its edit never ran?\n- Is the winner rule relative to the cohort? Or is the fitness vector missing dimensions?\n- Untried levers: bucket and probing structure, and a frequency signal that the eviction decision actually uses.\n\n## Per-policy notes\n\n- Current incumbent: cycle 3 v002 (one-bit unconditional second-sighting filter), built on the cycle 1 v001 MRU promotion. It appears to be a local optimum for admission rules.\n- Promotion/placement: MRU on hit is good. Further placement variants are exhausted on workload-a.\n- Victim selection: the furthest-from-MRU scan tied its parent. This is inconclusive until it is confirmed live.\n- Probing/stash: widening the neighbourhood cost a little hit-rate. Stash-eligibility relaxation was noise.\n- Admission: the simple one-bit filter beat all the more elaborate variants.\n"
}
```

### 04:23:37 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle output dir, using the four fixed sections in order. I built it only from the four cycle digests, without reading any previous insights file or code.

- **Recurring wins:** The two wins were MRU placement on hit-promotion (cycle 1) and the one-bit second-sighting admission filter (cycle 3). Both came from deciding what to admit or promote, not where entries sit.
- **Recurring failures:**
  - Placement tweaks added nothing, and extra movement (swaps, wider probes) hurt hit-rate.
  - Every refinement of the admission filter lost hit-rate.
  - Several variants matched their parent's hit-rate exactly, which may mean their edits never ran.
  - "Winner" sometimes meant best of a weak cohort rather than a real gain.
- **Open questions:** Whether the wins hold beyond workload-a and the current cache size. Per-op-time and metadata-size have never been measured. I also listed whether the exact-tie edits are live code, and how the winner rule works.
- **Per-policy notes:** The incumbent is the cycle 3 filter on top of the cycle 1 MRU promotion. The notes then cover promotion/placement, victim selection, probing/stash and admission.

I kept it qualitative, with no invented numbers.


---

- finished: 2026-10-21T04:23:38+00:00
- duration: 13.74s
- status: ok
- input_tokens: 4
- output_tokens: 1485
- cache_read_tokens: 36455
- cache_write_tokens: 8033
- cost_usd: 0.051052
