# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008
- allow_edits: True
- started: 2026-10-28T15:26:51+00:00

## timeline

### 15:26:55 tool: Bash

```json
{
  "command": "for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 15:26:57 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v001/insight.md"
}
```

### 15:26:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v002/insight.md"
}
```

### 15:26:58 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/variants/v003/insight.md"
}
```

### 15:26:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H01.md"
}
```

### 15:27:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H02.md"
}
```

### 15:27:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/hypotheses/H03.md"
}
```

### 15:27:05 thinking



### 15:27:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0008/cycle-insight.md",
  "content": "# Cycle 0008 Insight\n\n## Cohort outcome\n\nAll three variants built on cycle 0007 v001 (GCLOCK ceiling 7 + recent bit + halving decay, floored probation) and kept reset-to-0. v001 used linear decay in small main regions only. v002 added a one-shot \"veteran\" grace bit for blocks hit in main. v003 stopped counting probation hits that arrive in a correlated-reference window as promotion credit. v003 is the Winner: it gave the lineage's largest single-cell gain since probation was added (web_0 at the large size) and nudged hm_0 up at both sizes. Its only weak cell was web_0 at the small size, which was flat to slightly down. v001 and v002 both helped and hurt nothing.\n\n## Patterns\n\n- For the first time, a cohort had no real regressions. All three changes were safe, and each worked on a different path: v001 on small-main decay, v002 on the order of main's counter-0 victims, v003 on the probation promotion signal. They do not overlap, so they are candidates to combine, not alternatives.\n- The levers depend on scale, and each acts at only one size. v001 changed only the 64-slot main (the large cells were bit-identical). v002 and v003 mattered mainly in large buckets, because the grace visit and the window are both too short to matter in small ones. Changes should be sized in absolute slots per region, not as one rule or ratio for every size.\n- Tightening admission keeps winning. v003 is the fourth tightening of who reaches main to succeed (after probation, the depth floor and reset-to-0). It is also the first change to the promotion signal itself that worked. Burst re-references on web_0 were a meaningful share of the wasted counter-0 promotions that cycle 0007 identified.\n- web_0 gains more from main-side history than expected. v002's veteran bit helped web_0 at the large size more than hm_0, which refutes the idea that web_0's decayed blocks are mostly stale.\n- hm_0's large-size gap is still mostly unexplained. Recency (cycle 0007), victim ordering (v002) and promotion filtering (v003) each moved it only a little. Ranking inside main now looks nearly used up for hm_0.\n- Small buckets reward finer ranking. v001's recent bit plus linear counter confirmed cycle 0007's tier-resolution diagnosis without hurting web_0. Note that v002's insight blames the small-bucket cost on \"v001\", but it means cycle 0007 v001, not this cycle's v001.\n\n## Open questions\n\n- Does widening v003's window in 128-slot buckets (for example an absolute floor of about 16 slots) bring the web_0 gain to the small size, and at what width does hm_0's real reuse start being filtered?\n- Do v001, v002 and v003 stack? They act on different paths and sizes, but v002's grace bit and v003's filter both target counter-0 promotions in large main regions, so their gains may overlap.\n- What drives hm_0's remaining large-size gap? It is likely long-distance reuse that probation never sees.\n\n## Continuation\n\nMake cycle 0008 v003 the parent. First try folding in v001's size-gated linear decay, which should be close to free because it touches only small buckets. Also widen the correlated-reference window in small buckets using an absolute floor. Test v002's veteran bit on top of v003 as a separate variant, because the two may overlap. Do not revisit probation depth, hot-main admission gates, higher ceilings, ghost admission or non-zero promotion counters. Per-op-time is still unmeasured.\n"
}
```

### 15:27:26 thinking



### 15:27:27 chat

I've written `cycle-insight.md` for cycle 0008. In short: v003 wins, and for the first time no variant in the cohort regressed. All three variants built on cycle 0007 v001.

- **v003 (winner):** probation hits that come right after insertion no longer count toward promotion. This filters out burst re-references and gave the largest single-cell gain since probation was added (web_0 at the large size). hm_0 rose slightly at both sizes. web_0 at the small size was flat to slightly down, because the window there is only 4 slots and too narrow to catch bursts.
- **v001:** small main regions go back to linear decay, which gives finer ranking. This fixed hm_0 at the small size, and web_0 at the small size edged up. The large-bucket cells were bit-identical to the parent, as intended.
- **v002:** a one-shot "veteran" grace bit for blocks hit while in main. It helped web_0 at the large size more than hm_0, which was unexpected. hm_0 at the large size only just cleared its bar.

**Patterns I drew out:**
- Each change works at only one bucket size and on a different part of the cache, so the three could be combined rather than treated as alternatives.
- Making it harder to get into main keeps winning, and v003 is the first change to the promotion signal itself that worked.
- hm_0's gap at the large size is still mostly unexplained. Ranking inside main looks nearly used up.

**What the Continuation section tells the next cycle:**
- Make v003 the parent and add v001's decay change, which should cost nothing because it only affects small buckets.
- Widen v003's window in small buckets with a fixed minimum width.
- Test v002's veteran bit on top of v003 as a separate variant, since the two may overlap.
- Leave out everything retired in earlier cycles (probation depth, gates triggered by a hot main, higher ceilings, ghost admission, non-zero promotion counters).
- Note that per-op-time is still unmeasured.

v002's own insight says "v001" when it means cycle 0007 v001, not this cycle's v001. I corrected that in the cycle insight so later cycles don't misread it.


---

- finished: 2026-10-28T15:27:27+00:00
- duration: 36.16s
- status: ok
- input_tokens: 8
- output_tokens: 3080
- cache_read_tokens: 92731
- cache_write_tokens: 18100
- cost_usd: 0.224978
