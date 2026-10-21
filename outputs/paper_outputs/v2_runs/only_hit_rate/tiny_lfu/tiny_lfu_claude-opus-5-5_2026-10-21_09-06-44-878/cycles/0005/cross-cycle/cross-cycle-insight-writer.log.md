# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:30:20+00:00

## timeline

### 09:30:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Better estimate quality is the only lever that gave a clearly large hit-rate gain (a wider sketch in c3). Every other win in five cycles was marginal.\n- Lifetime frequency is the right signal for workload-a. Lineages that keep long-run counts intact always do at least as well as ones that add freshness.\n- Lineage stacking works: each cycle's single Winner (fast hash → sampled victims → wide sketch → dedup → SLRU) is a sound base for the next one, and the accumulated changes are compatible.\n- Making counts honest (deduplicating the get-miss-then-put double increment) gave a small but real gain once collision noise was gone.\n- The sketch is over-provisioned. A narrower sketch with smaller counters plus conservative update left every decision unchanged, so it is a likely memory win with no hit-rate cost.\n\n## Recurring failures\n\n- Sketch aging/halving lost or stayed flat in every form tried (short period with conservative update, long period alone, very long period on a clean sketch). On this trace it throws away useful history.\n- Changing the contest rule (tie-admission, `>=`) hurt: on ties it mostly swaps one cold key for another. Keep strict `>` with keep-victim-on-tie.\n- Widening or deepening the victim sample hurt. Without aging, the lowest estimate marks the youngest resident, not the coldest, so a wider sample evicts recent useful keys.\n- Window restructuring did nothing, whether as a static 1% window or an adaptive hill-climber. The naive climber drifts toward an LRU-like region, which this LIRS-style trace punishes.\n- Structural tweaks on clean estimates (SLRU split) mostly duplicate what min-estimate victim sampling already does, so gains stay marginal.\n- Accuracy fixes don't stack: width and conservative update correct the same collision error.\n- For five cycles running, per-op-time and metadata-size have gone unscored, so speed- and memory-oriented variants cannot win on what they actually improve.\n\n## Open questions\n\n- Once per-op-time and metadata-size are scored, which combination dominates (SLRU, narrow conservative-update sketch, hash-cached fast path), and what does SLRU's per-hit list work cost?\n- How small can the sketch get (width, counter size) before hit-rate moves?\n- Has workload-a reached a ceiling for TinyLFU-style policies, or is something left in the probation ratio or an admission window in front of SLRU?\n- Does the plateau and the \"aging is bad\" result generalize beyond workload-a? Every conclusion so far rests on a single trace.\n- A new newcomer still ties a resident that was accessed once, and loses. Is there a cheap fix for this that doesn't reintroduce the noise of tie-admission?\n\n## Per-policy notes\n\n- TinyLFU admission contest: strict `>` with keep-victim-on-tie is settled. The bottleneck was estimate quality, not the rule itself.\n- Count-min sketch: wide (or narrow plus conservative update) with dedup is the current best. Never-aging counters are correct for this trace. Fast early-exit shortcuts (estimate ≤ 1) depend on there being no aging and must be re-derived if decay is ever added.\n- W-TinyLFU window: neither fixed nor adaptive helped. Deprioritize.\n- SLRU probation/protected: a marginal win that overlaps with victim sampling. Its value now depends on per-op cost, which is still unmeasured.\n- Sampled-victim eviction: a modest default sample of the oldest entries helps a little. Widening it is exhausted.\n"
}
```

### 09:30:40 chat

I wrote `cross-cycle-insights.md` in the cycle-0005 cross-cycle directory, built only from the five-cycle ledger digest, with the four required sections in order.

The main patterns it records:

- **What paid off:** Cleaner frequency estimates gave the only large hit-rate gain (the wider sketch in cycle 3). Every other win was marginal. Each cycle's single winner stacked cleanly on the one before.
- **What keeps failing:**
  - Aging the sketch, in every form tried.
  - Changing the admission contest's tie rule.
  - Widening the victim sample.
  - Window restructuring, fixed or adaptive.
  - Combining fixes that correct the same estimate error.
- **The blocking issue:** For five cycles in a row, the evaluation scored only hit-rate. Per-op time and metadata size were never measured, so the variants built to be faster or smaller can't win on what they actually improve.
- **Open questions:**
  - Which combination of current winners comes out ahead once speed and memory are scored.
  - How small the sketch can get before hit-rate drops.
  - Whether this trace has hit its ceiling for this family of policies.
  - Whether a new key that ties a resident accessed once can be admitted cheaply, without the noise of changing the tie rule.
  - Whether any of these conclusions hold beyond workload-a, since every result so far comes from that one trace.

The per-policy notes record the settled choices for each component (admission contest, sketch, window, protected/probation split, victim sampling). They also flag that the speed shortcuts added in cycle 5 assume the sketch never ages and must be redone if aging is ever added.


---

- finished: 2026-10-21T09:30:41+00:00
- duration: 20.94s
- status: ok
- input_tokens: 4
- output_tokens: 1901
- cache_read_tokens: 37120
- cache_write_tokens: 9801
- cost_usd: 0.123868
