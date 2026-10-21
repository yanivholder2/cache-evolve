# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010
- allow_edits: True
- started: 2026-10-21T05:17:02+00:00

## timeline

### 05:17:06 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 05:17:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v001/insight.md"
}
```

### 05:17:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v002/insight.md"
}
```

### 05:17:09 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/variants/v003/insight.md"
}
```

### 05:17:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H01.md"
}
```

### 05:17:10 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H02.md"
}
```

### 05:17:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/hypotheses/H03.md"
}
```

### 05:17:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-07-49-975/cycles/0010/cycle-insight.md",
  "content": "# Cycle 0010 Insight\n\n## Cohort outcome\n\nAll three variants were built on cycle-0009 v002 (halved recency-gated ghost). v001 shrank the ghost again, to a quarter. v002 sent untouched demoted normal-tier victims to stash probation instead of MRU. v003 sent very recent ghost hits straight to the target bucket. v001 won by holding hit-rate within noise (slightly up) at a smaller ghost size. v002 lost heavily. v003 was neutral.\n\n## Patterns\n\n- The ghost-size floor is lower than expected. A quarter-size direct-mapped ghost still loses no hits, so the age gate, not table size, is still the limit. Two shrinks in a row each came out slightly positive, which looks like noise or the cheap removal of stale ghosts, not lost evidence.\n- The \"filter protected admission\" rule from 0004–0009 has a clear limit. Demotion to stash MRU is a second chance that pays off, not a leak. Keys that earned normal-tier residency carry reuse evidence even when they go untouched, likely because their reuse distance is longer than their stay in the normal tier. This is the first large hit-rate loss in several cycles, and it came from filtering a route that was already near its optimum.\n- Routing between the stash and the target is close to zero-sum. Admitting a key straight to target demotes a resident back to stash MRU, so the stash slot it frees is filled again at once. Very recent ghost hits promote reliably anyway, so skipping probation changes only when they arrive, not whether they survive.\n- Every admission route into protected space now appears tuned on this trace: ghost recency, BIP, fresh-key doorkeeper, tier ratio, demotion placement, and direct-to-target. The only gains left are in metadata, and those are still unmeasured. For the eighth cycle running, writers could not inspect source or instrumentation, and evaluation is one LIRS trace at one size, hit-rate only.\n\n## Open questions\n\n- Where is the actual ghost-size floor (an eighth, a sixteenth)? Is it set by collisions or by the epoch gate?\n- Did v003's age ≤ 1 branch actually fire, or mostly fall back? Unchanged results look the same as a no-op.\n- Did v002's \"hit in normal tier\" flag track exactly what it was meant to, or did reused state misclassify hot keys?\n\n## Continuation\n\nBuild on v001 (quarter-size ghost). Drop demotion-placement filters and direct-to-target admission. Both are settled as no better than the stash-mediated design. Further ghost shrinking is worth trying only as a single step to an eighth. Hit-rate mechanisms on this trace and size are exhausted. The next cycle should treat reporting metadata-size, per-op-time and branch-fire counters, and a second trace or cache size, as prerequisites rather than proposing new admission or placement tweaks.\n"
}
```

### 05:17:31 thinking



### 05:17:31 chat

I've written `cycle-insight.md` for cycle 0010.

**Cohort result:** v001 won. It cut the ghost table to a quarter of the cache and hit-rate stayed within noise, ending slightly up.
- **v002** lost heavily. It sent demoted keys that were never hit in the normal tier to the low-priority end of the stash instead of the top. Those keys tend to be reused soon after demotion, so they were evicted before that reuse arrived.
- **v003** made no measurable difference. It sent very recent ghost hits straight to the normal tier, skipping the stash. Each one pushed a normal-tier key back into the stash, which filled the slot it had just freed.

**What the insight says:**
- Since cycle 0004, every win came from tightening what gets into protected stash space. v002 is where that stops working: putting demoted keys back at the top of the stash is a second chance that pays off.
- Every route into protected space now looks tuned on this one trace and cache size. The only gains left are metadata savings, and nothing measures those yet.
- The continuation tells the next cycle to build on v001, and to drop demotion-placement filters and direct-to-normal-tier admission. It allows at most one more shrink, to an eighth of the cache. Before any new mechanisms, it asks for metadata-size, per-op-time and branch-fire counts to be reported, plus a second trace or cache size.

**Caveat:** this is the eighth cycle in which no insight writer could see the code changes. Two results are unconfirmed:
- v003's unchanged hit-rate could mean its new branch never fired.
- v002's loss could be partly a bug, if it reused an existing bit that doesn't track exactly "hit while in the normal tier".


---

- finished: 2026-10-21T05:17:32+00:00
- duration: 29.91s
- status: ok
- input_tokens: 8
- output_tokens: 2403
- cache_read_tokens: 85347
- cache_write_tokens: 14090
- cost_usd: 0.17834
