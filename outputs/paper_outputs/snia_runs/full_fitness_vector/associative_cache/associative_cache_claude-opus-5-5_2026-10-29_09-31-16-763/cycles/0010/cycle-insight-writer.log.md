# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010
- allow_edits: True
- started: 2026-10-29T13:05:21+00:00

## timeline

### 13:05:24 tool: Bash

```json
{
  "command": "for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 13:05:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v001/insight.md"
}
```

### 13:05:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v002/insight.md"
}
```

### 13:05:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/variants/v003/insight.md"
}
```

### 13:05:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H01.md"
}
```

### 13:05:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H02.md"
}
```

### 13:05:28 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/hypotheses/H03.md"
}
```

### 13:05:33 thinking



### 13:05:45 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-29_09-31-16-763/cycles/0010/cycle-insight.md",
  "content": "# Cycle 0010 Insight\n\n## Cohort outcome\n\nTwo variants built on the cycle 0009/v003 single-hash base. v001 stacked top-eighth lazy promotion onto it. v002 removed the `Box<dyn EvictionPolicy>` call from the ClassicLRU promote path. v003 went back to the older 0007/v003 base and shrank `Item` from 24 to 16 bytes by moving policy state into the existing `tags` buffer. All three are Winners. v002 is the clean win: put got much faster, get got slightly faster, and hit rate was bit-identical. v003 moved metadata-size for the first time since cycle 4, but both ops got slower at p50 for reasons nobody can explain yet. v001 cut put time but did not help get.\n\n## Patterns\n\n- The put-update path had more hidden cost than anyone expected. Both v001 and v002 got their largest gain on put, which neither hypothesis targeted. Both skip the same work: the virtual promote call plus the separate `tags` fix-up that `put_lip` reaches through `get_from_position`. Removing dispatch helped put far more than it helped get.\n- On get, in-bucket promotion is now close to the floor. With SipHash gone, devirtualisation gave only a small get gain. Lazy promotion gave the smallest measurable p50 step and made p99 worse, probably because its data-dependent branch runs on every hit. The two cycle-9 savings did not add up on get.\n- An exact-semantics fast path beats one that changes semantics. v002's \"already at back → return\" check keeps the order unchanged and has no hit-rate cost. v001's top-eighth window cost a little hit rate on the large-cache cells and added a branch that hurt the tail.\n- Shrinking `Item` works structurally. ClassicLRU order stayed exact, the bucket stride stayed at 80, and no field was added. But the expected latency gain from moving smaller items did not appear. The slowdown is most likely codegen fallout from widening the dynamic trait signature, or drift. This is inferred, not shown.\n- Hit-rate and stride invariants held again for every mechanical rewrite. As in cycles 7–9, single-run latency moves on unchanged paths still need same-session confirmation.\n\n## Open questions\n\n- Does v003's regression disappear once the lip path no longer goes through the dynamic call, so that the widened signature is off the hot path? Or does it disappear in a same-session A/B?\n- On a devirtualised base, do 16-byte items speed up the put-miss fused rotation and insert memmoves, which are now put's main remaining cost?\n- What dominates get now: the full-key compare, or bucket indexing (`%` vs mask)?\n\n## Continuation\n\nAdopt 0009/v003 plus v002's devirtualised `lip_promote` as the new base. Then port v003's 16-byte `Item` and side buffer onto that base. This also tests whether the regression came from the dynamic call, and it should be judged by repeated same-session runs against the base. Drop top-eighth lazy promotion and the planned `n/4` window probe. Drop any further get-side promotion work. Next, profile the full-key compare and bucket indexing for get, and the miss/insert memmoves for put. Keep the policy frozen.\n"
}
```

### 13:05:50 chat

I've written the cycle 0010 insight to `cycles/0010/cycle-insight.md`. All three variants are Winners, but only v002 is a clean speed win.

- **v002 (removed the virtual call from the promote path):** put got much faster, get got slightly faster, and hit rate didn't change at all. This is the main result of the cycle.
- **v001 (lazy promotion stacked on the cycle 0009 single-hash base):** put got faster, but get barely improved at p50 and got worse at p99. Hit rate dropped slightly on the large caches, still inside the allowed band. The two cycle 0009 savings did not add up on get.
- **v003 (smaller `Item`, 24 → 16 bytes):** metadata-size dropped for the first time since cycle 4, and hit rate didn't change. But get and put both got clearly slower, and the code change doesn't explain why. The variant's own insight guesses the larger virtual-call signature or measurement drift, but that's unconfirmed.

The main pattern: v001 and v002 both got their biggest gain on put, which neither hypothesis targeted. Both skip the same work on the put-update path, which turned out to hide more cost than expected. On get, moving items within a bucket is no longer worth optimising.

The Continuation section tells the next cycle to:
1. Use the cycle 0009/v003 base plus v002's change as the new base.
2. Move v003's smaller `Item` onto that base. This also tests whether its slowdown came from the virtual call, and should be checked with repeated runs in the same session.
3. Drop lazy promotion and the planned wider-window probe.
4. Profile the full-key compare and bucket indexing for get, and the miss/insert memory moves for put.


---

- finished: 2026-10-29T13:05:51+00:00
- duration: 30.02s
- status: ok
- input_tokens: 8
- output_tokens: 3035
- cache_read_tokens: 91076
- cache_write_tokens: 17854
- cost_usd: 0.221779
