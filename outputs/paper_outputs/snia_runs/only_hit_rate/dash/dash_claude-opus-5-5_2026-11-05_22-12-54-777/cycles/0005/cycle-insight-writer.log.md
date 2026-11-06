# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005
- allow_edits: True
- started: 2026-11-05T22:48:24+00:00

## timeline

### 22:48:28 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 22:48:29 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/variants/v001/insight.md"
}
```

### 22:48:30 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/variants/v002/insight.md"
}
```

### 22:48:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/variants/v003/insight.md"
}
```

### 22:48:31 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/hypotheses/H01.md"
}
```

### 22:48:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/hypotheses/H02.md"
}
```

### 22:48:32 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/hypotheses/H03.md"
}
```

### 22:48:44 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0005/cycle-insight.md",
  "content": "# Cycle 0005 Insight\n\n## Cohort outcome\n\nAll three variants built on cycle 0004's v002 (64-slot ghost ring, admission at counter 1). Each one isolated a single factor behind the ghost's limited reach on hm_0. v001 replaced the ring with a direct-mapped ghost table sized to the segment's capacity (reach). v002 stored the dropped counter and re-admitted at remembered+2, using 12-bit fingerprints (bonus strength). v003 removed the bucket-wide halving on stash hits (ageing drain). v001 was the Winner: it gave a clear gain on small-size hm_0, the largest ghost gain so far, with no regression anywhere. It also made ghost lookup O(1) instead of a per-`put` scan.\n\n## Patterns\n\n- Reach is what limits the ghost, not bonus strength. A cache-sized ghost clearly beat the ring. The stronger bonus helped only the same small-hm_0 cell, and only a little. Stash victims come from the low-counter tail, so the \"remembered\" count is usually near zero anyway.\n- Ghost gains stay concentrated in small-size hm_0, the same cell where the frequency filter has always done the most. At the large size, more reach and a bigger bonus both did nothing. The reuse still lost there is either beyond one cache's worth of drops or not saved by a single admission step.\n- web_0 stays inert to ghost reach. It reacted only to v002's narrower fingerprint, with a small dip from false matches or stickier admissions. Fingerprint width is a real cost, and scan-heavy traces are where it shows.\n- Halving on a stash hit is not an incidental drain. It is the stash's only ageing, and it also limits how high a counter a re-promoted item carries back into target. Removing it hurt every cell, the large size most. This is the third cycle confirming that counters must keep ageing, and that Front landing plus high counters makes items stick in target.\n- Per-op-time and metadata-size are still missing from the fitness vector, now for five cycles. The Winner's cost advantage, a scan replaced by an O(1) lookup, is argued from the code and has not been measured.\n\n## Open questions\n\n- Does a stronger bonus add on top of v001's cache-sized ghost, given that the ghost now fires more often? If it does, it needs to keep full-width fingerprints.\n- What reaches the large-size hm_0 reuse: a ghost larger than the cache, or a different admission rule?\n- Would ageing the stash at a different point (on insertion, or resetting/halving a demoted item's counter on entry into the stash) protect ghost credits without v003's stickiness?\n- Is there any lever at all for web_0?\n\n## Continuation\n\nMake cycle 0005 v001 (direct-mapped, capacity-sized ghost) the new parent. Next, try a bonus of 2 on it without cutting fingerprint bits, and try a ghost sized above capacity (for example 2×) for the large-size hm_0 cells. Drop removing the stash-hit halving, and drop any change that narrows the fingerprint. If stash ageing is revisited, move it rather than remove it.\n"
}
```

### 22:48:50 chat

I wrote `cycle-insight.md` in the cycle 0005 output dir. The Winner was v001: it replaced the 64-slot ghost ring with a ghost table sized to the segment's capacity. That gave the largest ghost gain so far, on hm_0 at the small cache size, with no regression on any trace or size.

- **Main finding:** how far back the ghost remembers is what limits it, not how big the re-admission bonus is. v002's stronger bonus helped only the same small hm_0 case, and only a little.
- **What hurt:** v002 shortened the fingerprint, which caused a small dip on web_0. v003 removed the halving on stash hits, which turned out to be the stash's only ageing; removing it hurt every trace and size, the larger size most.
- **Still unmeasured:** per-op-time and metadata-size have now been missing for five cycles. v001's claim that it is cheaper, a single lookup instead of a scan on every `put`, comes from the code alone.
- **Continuation:** use v001 as the new parent. Next, try a bonus of 2 with full-width fingerprints, and a ghost larger than the cache (for example 2×) for the large hm_0 size. Drop removing the stash-hit halving and drop narrower fingerprints. If stash ageing is changed again, move it rather than remove it.


---

- finished: 2026-11-05T22:48:50+00:00
- duration: 26.00s
- status: ok
- input_tokens: 8
- output_tokens: 2338
- cache_read_tokens: 89858
- cache_write_tokens: 15849
- cost_usd: 0.191556
