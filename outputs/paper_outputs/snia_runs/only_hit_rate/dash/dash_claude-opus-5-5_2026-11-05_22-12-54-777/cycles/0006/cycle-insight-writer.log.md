# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006
- allow_edits: True
- started: 2026-11-05T22:55:29+00:00

## timeline

### 22:55:33 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006; for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 22:55:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/variants/v001/insight.md"
}
```

### 22:55:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/variants/v002/insight.md"
}
```

### 22:55:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/variants/v003/insight.md"
}
```

### 22:55:36 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/hypotheses/H01.md"
}
```

### 22:55:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/hypotheses/H02.md"
}
```

### 22:55:37 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/hypotheses/H03.md"
}
```

### 22:55:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0006/cycle-insight.md",
  "content": "# Cycle 0006 Insight\n\n## Cohort outcome\n\nAll three variants built on cycle 0005's v001 (a direct-mapped ghost sized to the cache, admitting at counter 1). Each one isolated a different explanation for why large-size hm_0 stays stuck. v001 raised the ghost-hit credit to 2 (bonus strength). v002 doubled the ghost table (reach). v003 halved an item's counter when it was demoted into the stash (where ageing happens). v003 was the Winner. It was the only variant with a small, consistent hm_0 gain at both sizes, including the first movement at the large size, and web_0 was flat to noise-level negative.\n\n## Patterns\n\n- Ghost reach and ghost bonus strength are now both used up. A credit of 2 barely moved small-size hm_0, because a credit of 1 already beats the mostly counter-0 stash tail. A 2× table did nothing at the large size and slightly hurt the small size, because older returners diluted the bonus. Neither the bonus nor reach is what limits large-size hm_0.\n- What helped was changing who the ghost-credited arrivals compete against, not making the credit itself bigger. Taking away the stale target-tier counts that demoted items carried into the stash helped. This is the fourth cycle in which counter ageing turned out to be the active lever. It is also the first time ageing was added at a point where no hit is penalised, and that is why it was safe, unlike removing the stash-hit halving.\n- Gains still depend on the ghost firing. That is why hm_0 responds and web_0 does not. web_0's only reactions this cycle were noise-level dips: demoted items returning to target with less protection (v003), and over-credited scan keys (v001).\n- The effects are getting smaller each cycle. Every lever in this family (window, ageing, ghost reach, bonus, ageing point) now yields small gains, which suggests the current structure is close to its limit.\n- Per-op-time and metadata-size are still not in the fitness vector, now for six cycles. v002's extra metadata was reported only as an estimate.\n\n## Open questions\n\n- Do other tier boundaries also benefit from moving ageing there? For example: halving on stash→target promotion, or ageing applied inside the normal tier's tail scan.\n- Does large-size hm_0 lose reuse through exit paths that never reach the ghost (keys leaving other than by stash drops)?\n- Would a stronger demotion decay (a quarter, or a reset) help hm_0 more, or start hurting web_0 at the large size?\n- Is there any lever at all for web_0, perhaps scan detection or admission filtering on the normal tier rather than the stash?\n\n## Continuation\n\nMake cycle 0006 v003 (halve on demotion into the stash) the new parent. Drop further ghost-size and ghost-bonus tuning; both are ruled out. Next, test where ageing is applied at other boundaries (promotion back into target, normal-tier eviction), and check whether hm_0's lost keys leave through paths the ghost never records. Given the shrinking gains, consider one structurally different hypothesis aimed at web_0, and push again for per-op-time and metadata-size to be measured.\n"
}
```

### 22:55:57 thinking



### 22:55:57 chat

I wrote `cycle-insight.md` for cycle 0006 in the output dir.

- **Outcome:** v003 is the Winner. It halves an item's counter when the item is demoted into the stash. It was the only variant that gave a small, consistent gain on hm_0, at both cache sizes. That includes the first movement ever on the large-size hm_0 cell. web_0 was flat or dipped at noise level.
- **Ruled out:** The two other changes both failed. Raising the ghost credit to 2 (v001) barely helped, and doubling the ghost table (v002) did nothing at the large size and slightly hurt the small size. So neither ghost reach nor bonus strength is what limits large-size hm_0.
- **Pattern:** For the fourth cycle running, how counters age is what matters. Ageing helped this time because it was added at a point where no hit gets penalised. Gains across this whole family of changes keep getting smaller, which suggests the current design is near its limit.
- **Continuation:** Make v003 the new parent and stop tuning ghost size and bonus. The next cycle should:
  - try ageing at other boundaries, such as when items are promoted back into target;
  - check whether hm_0's lost keys leave through paths the ghost never records;
  - try one structurally different idea aimed at web_0, which still has no lever that moves it.

Per-op-time and metadata-size have now gone unmeasured for six cycles, and the insight flags this again.


---

- finished: 2026-11-05T22:55:58+00:00
- duration: 28.85s
- status: ok
- input_tokens: 8
- output_tokens: 2416
- cache_read_tokens: 89488
- cache_write_tokens: 15158
- cost_usd: 0.187514
