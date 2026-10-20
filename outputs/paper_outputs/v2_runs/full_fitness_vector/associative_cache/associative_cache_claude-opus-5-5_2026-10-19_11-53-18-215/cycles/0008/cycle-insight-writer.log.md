# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008
- allow_edits: True
- started: 2026-10-20T09:43:13+00:00

## timeline

### 09:43:22 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 09:43:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v001/insight.md"
}
```

### 09:43:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v002/insight.md"
}
```

### 09:43:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/variants/v003/insight.md"
}
```

### 09:43:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H01.md"
}
```

### 09:43:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H02.md"
}
```

### 09:43:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/hypotheses/H03.md"
}
```

### 09:43:33 thinking



### 09:43:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0008/cycle-insight.md",
  "content": "# Cycle 0008 Insight\n\n## Cohort outcome\n\nAll three hypotheses started from cycle 0007 v001 (tag-first find, single hash, 7/8 cap), and each changed one separate axis:\n- v001 merged cycle 0007 v003's 15/16 cap.\n- v002 laid out the bucket hot/cold, with tags inline and ghost/policy state boxed.\n- v003 put ghost-miss cold keys at probation LRU instead of mid-probation.\n\nv001 and v003 are Winners on merit. v001 gained hit rate with no latency cost and a better put tail. v003 is the first hit-rate gain on this lineage that leaves the cap alone, and it also made put faster. v002 did not lower get latency and only moved put cost around, so it is a Winner by default.\n\n## Patterns\n\n- The best changes remove work. v003's LRU entry drops a rotate, and v001's smaller probation shortens the cold-insert rotate. Both improved hit rate and the put tail at once. Changes that add indirection to save memory touches (v002's cold box and inline/heap branch) cost as much as they save.\n- The cap carried forward cleanly. The merge reproduced the hit-rate target exactly on top of tag-first lookup, which confirms that lookup and replacement are orthogonal. The worry about protected-rotate cost at 15/16 did not show up.\n- Get latency no longer depends on struct locality. After tag-first scanning, the working set is likely cache-resident, so get now costs instructions and data movement per hit, not cache lines.\n- Admission is now the hit-rate axis with room left. Where a history-less key enters decides whether it or the just-demoted protected item is evicted next. The lever only works when probation has at least 2 slots, though, and the 15/16 cap shrinks probation. v001 and v003 therefore pull against each other: at 16-slot buckets, the merged cap leaves a 1-slot probation, where LRU and middle entry are the same.\n\n## Open questions\n\n- Do v001 and v003 stack, or does 15/16's thinner probation (one slot at 16-slot buckets) cancel LRU insertion there? Would a probation floor of 2 at all sizes ≥16 be a better point once cold keys enter at LRU?\n- With one-hit-wonders now evicted first, does the cap even need to be this large? Could a wider probation with LRU entry beat 15/16 on hit rate?\n- Would a stronger admission signal, such as a frequency sketch or a ghost-hit count, beat a single ghost bit?\n- The miss-path costs are still unmeasured: the linear ghost scan and whole-bucket insert/remove shifts.\n\n## Continuation\n\nMerge v001 and v003: 15/16 cap plus cold keys at probation LRU, on the tag-first base. Then test the probation floor jointly with LRU entry, since the cap and admission now interact through probation width. Keep pursuing admission and ghost policy for hit rate. For latency, target per-hit work and miss-path data movement (non-linear ghost lookup, cheaper probation shifts). Drop bucket-struct layout and hot/cold splitting, and do not add indirection to the put path.\n"
}
```

### 09:43:52 chat

I wrote `cycle-insight.md` in the cycle 0008 output folder.

- **What worked:** v001 and v003 are Winners on merit.
  - v001 put the 15/16 protected cap on the tag-first base. It hit the hit-rate target with no latency cost and made the slowest puts faster.
  - v003 sends new keys with no history to the eviction end of probation, the short-term segment. It raised hit rate without touching the cap and also sped up put.
- **What didn't:** v002 reorganised the bucket's memory layout. Get latency didn't improve and put cost only shifted around, so it is a Winner by default only.
- **Pattern:** the changes that helped removed work. The one that added indirection to save memory accesses broke even at best.
- **Catch for the next cycle:** v001 and v003 work against each other. The 15/16 cap shrinks probation to one slot in 16-slot buckets. With one slot, where a new key enters doesn't matter, so v003's gain disappears there.
- **Continuation:** merge v001 and v003, then tune the minimum probation size together with the new entry rule. Keep looking for hit rate in which keys get admitted, and for latency in the miss path. Drop memory-layout work and anything that adds indirection to put.


---

- finished: 2026-10-20T09:43:54+00:00
- duration: 41.14s
- status: ok
- input_tokens: 8
- output_tokens: 2628
- cache_read_tokens: 87824
- cache_write_tokens: 16219
- cost_usd: 0.199909
