# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:44:08+00:00

## timeline

### 15:44:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy and mechanics are independent axes. Hit-rate comes only from the policy (segmentation, reference bit). Speed and size come from mechanics (hasher, clone removal, link layout). Mechanics changes that leave eviction order alone have preserved hit-rate in every cycle.\n- Trimming hashing and eviction-path work pays off most on put: removing the eviction key clone, a lighter hasher, and dropping the avalanche fold all helped put.\n- Segmentation reliably gives scan-resistance hit-rate. This is established.\n- Combining independent Winners (segmentation, fast hasher, cheap touch, move-based eviction) worked as a merge strategy.\n\n## Recurring failures\n\n- Shifting promotion/demotion work between operations does not remove it. Lazy demotion (put), reference-bit get (eviction walk) and similar moves only relocate the cost, usually onto the put tail.\n- Layout tweaks on the segmented policy (u32 links, packing the segment bit) bought size at best, not speed. They add widening, NIL checks and masking on every link access.\n- Partial-coverage promotion skips (head-adjacent relink skip) add a branch and save little, because few hits land there.\n- Bundling several changes (fold with layout, hasher with touch path and clone removal) blurs attribution. Authors cannot read diffs, so causes are inferred.\n- Infrastructure failures (a build rate limit in cycle 1) leave ideas untested. Do not read them as negative results.\n\n## Open questions\n\n- Do the fold-free hasher and the reference-bit get path compose, and do the gains add up?\n- Can the eviction walk be bounded or amortized (capping the referenced nodes passed per put) to fix put p99 without losing hit-rate?\n- Can the segment be stored implicitly (by list membership) so there is no masking?\n- Why did put slow in a variant that did not target it, and did the unexplained struct shrink hold up?\n- Does segmentation's hit-rate gain justify its latency cost on traces other than the scan-heavy one? No cycle has checked yet.\n- Is the u32 link speed benefit real at a larger working-set size?\n\n## Per-policy notes\n\n- Plain LRU: u32 links, a lighter hasher and a head-touch early return were pure wins, with smaller size and faster ops, and no hit-rate change.\n- Segmented LRU: gained hit-rate on scans, but its promotion/demotion relinking is the standing latency cost, mostly on the put tail. Its container also grows from the extra head/tail pair. The merged form (u32 links, Fx hasher, move-based eviction) is the working baseline, and the fold should stay removed.\n- Second-chance (CLOCK-style) reference bit: a small extra scan-resistance gain and a cheaper get, with the cost moved to the eviction walk and the put tail.\n"
}
```

### 15:44:29 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 `cross-cycle` output dir. It has the four fixed sections in order, built only from the four-cycle ledger digest.

- **Recurring wins:** hit-rate comes from the policy (segmentation, reference bit), while speed and size come from mechanics (hasher, clone removal, link layout). Trimming hashing and eviction-path work helps put most. Merging independent Winners has worked.
- **Recurring failures:** moving promotion or demotion work between `get` and `put` only relocates the cost, usually onto the put tail. Layout tweaks on the segmented policy buy size at best, not speed. Partial-coverage relink skips don't pay. Bundled changes blur attribution, and infrastructure failures leave ideas untested.
- **Open questions:** whether the fold-free hasher and the reference-bit get path compose, and whether the eviction walk can be bounded to fix put p99. Other threads: implicit segment storage, an unexplained put slowdown and struct shrink, hit-rate on traces other than the scan-heavy one, and whether u32 links help at larger working sets.
- **Per-policy notes:** separate notes for plain LRU, segmented LRU and the second-chance reference bit.

I didn't put any numbers in the file, as the task says.


---

- finished: 2026-10-20T15:44:30+00:00
- duration: 21.81s
- status: ok
- input_tokens: 4
- output_tokens: 1532
- cache_read_tokens: 37102
- cache_write_tokens: 8738
- cost_usd: 0.054471
