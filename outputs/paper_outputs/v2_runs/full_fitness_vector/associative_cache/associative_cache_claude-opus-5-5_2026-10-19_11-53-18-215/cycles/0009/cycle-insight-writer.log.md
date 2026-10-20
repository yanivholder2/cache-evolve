# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009
- allow_edits: True
- started: 2026-10-20T09:55:33+00:00

## timeline

### 09:55:37 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009; for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 09:55:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v001/insight.md"
}
```

### 09:55:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v002/insight.md"
}
```

### 09:55:40 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/variants/v003/insight.md"
}
```

### 09:55:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H01.md"
}
```

### 09:55:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H02.md"
}
```

### 09:55:42 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/hypotheses/H03.md"
}
```

### 09:55:44 thinking



### 09:55:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-19_11-53-18-215/cycles/0009/cycle-insight.md",
  "content": "# Cycle 0009 Insight\n\n## Cohort outcome\n\nEach variant tested one axis:\n- v001 stacked the 15/16 cap on cycle 0008 v003's LRU cold entry, with a probation floor of 2.\n- v002 added a per-bucket count-min frequency gate for admission.\n- v003 swapped the miss-path whole-bucket shifts for an in-place probation rotate, plus a branch-free ghost scan and an MRU early return in `hit()`.\n\nOnly v001 moved hit rate up, by a small amount, and it paid for it in get and put tail latency. v003 kept hit rate the same but was slightly slower. v002 lost hit rate and was slower. v002 and v003 are Winners by default, not on merit.\n\n## Patterns\n\n- The cap and LRU cold entry stack, but only just. The floor of 2 never took effect, because 0008 v003 already kept 2+ probation slots at 16–31. So v001 is really \"15/16 on buckets of 32+ slots\". The cap axis is close to saturated: making probation narrower now mostly buys longer rotates and more ghost-hit re-inserts on the tail.\n- An admission signal is only worth something if it can tell keys apart. The undersized, slowly aged sketch saturated, so nearly every key counted as \"seen\". Admission quietly fell back to MRU entry, which undid v003's one-hit-wonder protection, and the put path got heavier for nothing.\n- The miss-path memmove has now been measured, and it is not a cost. Shifting 16–32 sixteen-byte items on the minority of puts that are full ghost hits is cheap. Removing the shift saved nothing visible. The fixed extras that came with it (a ghost scan with no early exit, an MRU branch that rarely fires under 15/16) made both medians worse.\n- Across the cohort, every latency regression came from extra per-op work: branches, counter updates, a larger bucket footprint. None came from bytes moved. Get and put now cost instructions per op, not data movement.\n\n## Open questions\n\n- Would a sketch sized correctly help? That needs enough counters for the window size divided by the threshold, index bits taken from more of the tag, and a saturation check done before measuring.\n- Would a ghost-hit count, or a 2-bit ghost entry, give a better admission signal than a sketch, at lower cost?\n- Is v001's small hit-rate gain worth its tail cost, compared with 0008 v003's 7/8 cap at 32+?\n\n## Continuation\n\nUse cycle 0009 v001 as the hit-rate base, but treat the cap as finished: change neither the cap nor the probation floor any further. Drop miss-path memmove work, branch-free ghost scans and MRU short-circuits, because data movement on the miss path is not the bottleneck. For hit rate, retry admission only if the signal is checked to discriminate before it is measured (a correctly sized and aged sketch, or ghost-hit counts). For latency, cut fixed per-op instructions on the get-hit path rather than bytes moved.\n"
}
```

### 09:56:01 chat

I've written `cycle-insight.md` for cycle 0009. Only v001 is a Winner on merit; v002 and v003 are Winners by default because they didn't improve anything.

- **v001 (15/16 cap on top of cold keys entering probation at LRU):** hit rate went up slightly, but get and put both got slower in the tail. The floor of 2 probation slots never actually took effect, because the parent already kept 2+ slots at 16–31. So this was really just the 15/16 cap on buckets of 32+ slots. The cap axis looks close to used up.
- **v002 (frequency-count filter for admission):** hit rate went down and put got slower. The filter was too small for how slowly it aged its counts, so it saturated and let nearly every key in as if it had been seen before. That undid the protection against keys that are used only once.
- **v003 (in-place miss-path rotate, branch-free ghost scan, early return when a hit is already at MRU):** hit rate stayed the same and per-op time got slightly worse. Removing the whole-bucket shifts on the miss path saved nothing measurable. The added branches and the ghost scan that no longer stops early made both medians worse.

Across the cohort, every slowdown came from extra work per operation, not from data being moved.

The Continuation section tells the next cycle to:
- build on v001 and stop changing the cap and probation floor;
- drop miss-path shift work and the other small per-op tweaks;
- retry admission only with a signal that is checked to tell keys apart before hit rate is measured;
- go after per-op instruction cost on the get-hit path for latency.


---

- finished: 2026-10-20T09:56:02+00:00
- duration: 29.32s
- status: ok
- input_tokens: 8
- output_tokens: 2531
- cache_read_tokens: 87895
- cache_write_tokens: 16026
- cost_usd: 0.196439
