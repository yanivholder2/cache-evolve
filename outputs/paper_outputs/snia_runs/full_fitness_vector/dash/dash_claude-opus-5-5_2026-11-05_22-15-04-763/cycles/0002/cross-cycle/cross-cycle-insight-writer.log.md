# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-11-05T22:32:05+00:00

## timeline

### 22:32:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Hit rate improves with a two-hit gate on promotion from stash to the normal tier. This held in both cycles, and every cycle-0002 variant built on the gate still beat the baseline on hit rate.\n- Changes that keep proven or recently demoted items around longer reliably raise hit rate. Examples: sending promotions into free probing space instead of evicting a resident, and demoting normal-tier victims to the stash MRU end.\n- Large-size hm_0 is the most sensitive configuration in both cycles. It gains the most when re-referenced items are kept and loses the most when they are dropped. Use it as the first place to check any retention idea.\n- Reusing the existing unused per-item field (`lfu_counter`) for gate state cost no extra metadata. Prefer this over adding new fields.\n\n## Recurring failures\n\n- Tail latency: no variant has yet improved p99 in either cycle. Each hit-rate gain came with a small tail regression, for a different reason each time: the promote/demote cascade, longer lookups, and stash occupancy pressure.\n- `put` slowed down in both cycles and has never been attributed to stash fullness versus benchmark noise.\n- Skipping the MRU move on the first stash hit gave a slightly better p50, the same p99, and some lost hit rate. Treat this direction as closed.\n- In cycle 0001, the simulator build failed on Maven/Gradle 429 rate limits and two variants were never measured. Infrastructure failures can silently shrink a cohort.\n\n## Open questions\n\n- Do MRU demotion and probing overflow stack on top of the gate? Might probing overflow's fewer demotions offset the stash pressure that MRU demotion adds?\n- Can the third-bucket lookup cost of probing overflow be removed, for example through search order or migrating items back to target, while keeping its hit-rate gain?\n- Is the `put` p99 regression actually caused by stash occupancy? It needs a run that separates the effect from noise.\n- Can the latency of the second-hit cascade be reduced directly, for example with balanced target/probing promotion? Balanced promotion was proposed in cycle 0001 but has never been measured.\n\n## Per-policy notes\n\n- Dash (stash + normal tier):\n  - Current base: two-hit gate plus MRU demotion into stash. It has the strongest hit rate so far, at the cost of a small tail regression.\n  - The gate's latency cost comes from the second-hit cascade and lookup length, not from the first-hit path.\n  - Promotion placement (where promotions land) and demotion placement (where demotions land) are independent levers. Each helps hit rate on its own, and they have not been tested together.\n"
}
```

### 22:32:24 thinking



### 22:32:24 chat

I wrote `cross-cycle-insights.md` from the two-cycle ledger, using the four required sections in order.

- **Recurring wins:** The two-hit promotion gate has raised hit rate in both cycles. Keeping proven or recently demoted items around longer has also paid off: sending promotions into free probing space, and demoting evicted items to the most-recently-used end of the stash. Large-size hm_0 is the configuration most sensitive to these changes. Reusing the unused `lfu_counter` field for gate state added no metadata.
- **Recurring failures:** No variant in either cycle has improved p99 latency; every hit-rate gain came with a small tail regression. The `put` slowdown appeared in both cycles and has never been pinned on stash fullness versus benchmark noise. Leaving the item in place on its first stash hit didn't pay off and is marked as a dead end. In cycle 0001, two variants were lost to build rate-limit errors, so they were never measured.
- **Open questions:** Whether most-recently-used demotion and probing overflow add up when combined. How to remove the extra bucket lookup that probing overflow causes. What actually causes the `put` regression. Balanced promotion (proposed in cycle 0001) has still never been measured.
- **Per-policy notes:** The Dash base is now the two-hit gate plus most-recently-used demotion. The latency cost comes from the second-hit move between tiers and from longer lookups. Where promotions land and where demotions land are separate settings that haven't been tested together.


---

- finished: 2026-11-05T22:32:25+00:00
- duration: 19.97s
- status: ok
- input_tokens: 4
- output_tokens: 1536
- cache_read_tokens: 36000
- cache_write_tokens: 7058
- cost_usd: 0.094714
