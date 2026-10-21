# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004
- allow_edits: True
- started: 2026-10-21T09:29:59+00:00

## timeline

### 09:30:03 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004; for f in variants/v00*/insight.md hypotheses/H0*.md; do echo \"=== $f\"; cat \"$f\"; done",
  "description": "Read variant insights and hypotheses"
}
```

### 09:30:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v001/insight.md"
}
```

### 09:30:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v002/insight.md"
}
```

### 09:30:06 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v003/insight.md"
}
```

### 09:30:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H01.md"
}
```

### 09:30:07 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H02.md"
}
```

### 09:30:08 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H03.md"
}
```

### 09:30:11 thinking



### 09:30:23 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/cycle-insight.md",
  "content": "# Cycle 0004 Insight\n\n## Cohort outcome\n\nAll three variants came from cycle 3's v001 (ghost at half of capacity) and each tried a different way to filter the ghost-hit path into `main`:\n- **v001** shrank the ghost again, to a quarter of capacity. It won with a modest gain.\n- **v003** kept the ghost size and added a count-min frequency gate on ghost re-admission. Its gain was only marginal.\n- **v002** routed ghost hits into `small` and required two hits for cold keys. Its hit rate regressed.\n\n## Patterns\n\n- **Shrinking the ghost is still the best lever, but the gains are getting smaller.** Two halvings in a row have helped, and this one helped less than the last. The optimum is probably close to a quarter of capacity or a little below it. It has only been measured at one cache size.\n- **A ghost hit is a useful reuse signal once the window is short.** Cycle 2 called ghost hits misleading on long loops. With a short window, though, they mostly catch real reuse. v002 made ghost-hit keys prove reuse again inside the tiny probation queue and lost hit rate. v003's frequency gate found little left to filter.\n- **Requiring reuse while resident has hit a limit.** In earlier cycles, stricter admission kept paying off. Here it stopped: with a probation queue of about 10%, a two-hit bar or a forced pass through `small` evicts keys whose reuse gap is only slightly longer than that queue. Admission can be too strict as well as too loose.\n- **Gates on the ghost path leak.** Both v002 and v003 left keys a second route into `main` through `small`, the same leak cycle 3 found with the promotion bar. v003 also compared against `main.front()`, which is often not the entry that actually gets evicted. Any admission gate should compare against the real eviction victim.\n- **Measurement is still missing, for the fourth cycle running.** v001's smaller ghost probably saves memory. v003's sketch probably costs both memory and time, since it adds a table and extra hashing on every operation. Neither effect has been verified.\n\n## Open questions\n\n- Where does hit rate turn over: an eighth of capacity, or lower? Does the best ghost size hold at other cache sizes?\n- Was v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately. Its ghost bookkeeping fix, which stops stale entries erasing newer ones, is correct and should be kept.\n- Would TinyLFU-style admission against the real eviction victim, applied to all insertions and not only ghost hits, pay back the sketch's unmeasured cost?\n\n## Continuation\n\nUse this cycle's v001 as the parent. Run one more ghost-size probe at an eighth of capacity, at more than one cache size if possible, then freeze the ghost size. Port v002's sequence-number ghost fix by itself as a correctness change.\n\nDrop forced routing of ghost hits through `small`, and drop two-hit promotion bars for now. Drop ghost-path-only frequency gates. Bring the sketch back only as victim-based TinyLFU admission, and only once `per-op-time` and `metadata-size` can be measured. With the ghost size settling, run the long-postponed representation-only experiments next: the fingerprint ring and the hasher plus narrow-index graft. Getting the harness to measure those two dimensions is still the top priority.\n"
}
```

### 09:30:31 chat

I wrote `cycle-insight.md` for cycle 0004 in the output directory.

**Cohort outcome:** All three variants started from cycle 3's v001, and each tried a different way to filter keys that come back through the ghost into `main`.
- **v001 (Winner):** shrank the ghost from half to a quarter of capacity. Hit rate went up modestly.
- **v003:** added a frequency-sketch check on keys returning through the ghost. The gain was only marginal.
- **v002:** sent returning ghost keys into `small` and required two hits for new keys. Hit rate went down.

**Main patterns:**
- **Ghost size still helps, but less each time.** Two halvings in a row have raised hit rate, and this one raised it less than the last. The best size is probably near a quarter of capacity, but it has only been tested at one cache size.
- **Admission can now be too strict.** Earlier cycles kept gaining from stricter admission. Here, v002's extra hurdles inside the small probation queue (about 10% of the cache) threw away keys that were genuinely being reused.
- **Checks on the ghost path leak.** Keys rejected by v002 and v003 could still reach `main` through `small`. v003 also compared against an entry that often isn't the one actually evicted.
- **Memory and speed have now gone unmeasured for four cycles.** The smaller ghost probably saves memory, and v003's sketch probably costs memory and time, but neither is verified.

**Continuation for the next cycle:**
- Use this cycle's v001 as the parent.
- Try one more ghost size, an eighth of capacity, at more than one cache size if possible, then lock it in.
- Port v002's ghost bookkeeping fix on its own; it fixes a real bug and is worth keeping.
- Drop sending returning ghost keys through `small`, the two-hit bar, and checks applied only to the ghost path.
- Bring the sketch back only as TinyLFU-style admission compared against the real eviction victim, and only once memory and speed can be measured.
- Next, run the memory-layout experiments postponed since cycle 2.
- Fixing the harness to report memory and speed is still the top priority.


---

- finished: 2026-10-21T09:30:32+00:00
- duration: 33.00s
- status: ok
- input_tokens: 8
- output_tokens: 2929
- cache_read_tokens: 86472
- cache_write_tokens: 15740
- cost_usd: 0.201826
