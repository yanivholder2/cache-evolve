# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003
- allow_edits: True
- started: 2026-10-28T18:43:33+00:00

## timeline

### 18:43:37 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 18:43:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v001/insight.md"
}
```

### 18:43:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v002/insight.md"
}
```

### 18:43:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/variants/v003/insight.md"
}
```

### 18:43:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/hypotheses/H01.md"
}
```

### 18:43:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/hypotheses/H02.md"
}
```

### 18:43:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/hypotheses/H03.md"
}
```

### 18:43:46 thinking



### 18:43:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0003/cycle-insight.md",
  "content": "# Cycle 0003 Insight\n\n## Cohort outcome\n\nAll three variants built on cycle-0002 v001 (MRU-end stash demotion plus the two-hit gate). Each changed one aspect of the gate: v001 changed which items it applies to (demoted items return on one hit), v002 changed when it applies (skipped when the target has a free slot), and v003 changed where items land once they pass it (target MRU end instead of LRU end). The Winner is v003. Its gains came from both 16K cells and `hm_0` at 128K, while `web_0` at 128K stayed flat.\n\n## Patterns\n\n- Where a promotee lands mattered more than tuning the admission rule. With `Front` placement, an item that passed the gate became the target's next eviction victim, so the gate was partly defeating itself. Fixing placement paid off most where target slots are contested.\n- v001 disproved the cycle-0002 explanation. Removing the counter-reset penalty on demoted items barely moved `web_0` at 128K. The regression there comes from gating fresh puts when targets have room, not from the reset.\n- v002 is the only variant that recovered `web_0` at 128K. v003 gained in the cells v002 lost (16K) and stayed flat in the cell v002 fixed. The two levers cover opposite cells and look complementary.\n- Small caches kept penalizing any easier route into the target (v001's one-hit return, v002's free-slot shortcut). In tight caches, a one-hit item that takes the last slot pushes a cascade onto the next promotion. A free slot is not actually free when space is tight.\n- The variant insights again report only hit-rate. Per-op-time and metadata-size are still unverified.\n\n## Open questions\n\n- Does v002's occupancy shortcut stack on v003's `Back` placement, recovering `web_0` 128K without giving back the 16K gains?\n- Would a stricter room condition (more than one free slot, or an occupancy fraction) remove v002's 16K cost?\n- Is v003's flat `web_0` 128K cell caused by promotees jumping to probing on their next hit? Would requiring one more target hit before probing fix it?\n\n## Continuation\n\nBuild on v003. The top candidate is to add v002's occupancy-aware gate on top of it, preferably with a stricter room threshold so the 16K cells are protected. A second candidate is delaying a fresh promotee's move to probing. Drop the demoted-item counter lever (v001), since it is near-neutral. Get per-op-time and metadata-size measured before making any cost claims.\n"
}
```

### 18:44:01 chat

I wrote the Cycle 0003 insight to `cycles/0003/cycle-insight.md`.

- **What was tried:** all three variants built on cycle-0002 v001. Each changed one aspect of the two-hit gate:
  - v001 changed which items it applies to (demoted items get back on one hit).
  - v002 changed when it applies (it is skipped when the target has a free slot).
  - v003 changed where items land after passing it (the target's MRU end instead of its LRU end).
- **Winner:** v003. It gained at 16K and on `hm_0` at 128K. `web_0` at 128K stayed flat.
- **Main finding:** where a promoted item lands mattered more than the admission rule. Under the old placement, an item that passed the gate became the target's next eviction victim, so the gate was partly undoing itself.
- **Explanation ruled out:** v001 shows that resetting the counter on demotion was not what caused the `web_0` 128K regression. The real cause is gating fresh puts when the target has room.
- **Complementary levers:** v002 was the only variant that recovered `web_0` at 128K. It lost slightly at 16K, which is exactly where v003 gained.
- **Continuation:** build on v003 and add v002's occupancy check, ideally with a stricter room threshold to protect the 16K cells. A second option is to delay a newly promoted item's move to probing. Drop the demoted-item counter change, since it was close to neutral.

The variant insights again reported hit-rate only, so per-op-time and metadata-size are still unmeasured.


---

- finished: 2026-10-28T18:44:01+00:00
- duration: 27.79s
- status: ok
- input_tokens: 8
- output_tokens: 2428
- cache_read_tokens: 85264
- cache_write_tokens: 13829
- cost_usd: 0.176277
