# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-28T18:59:39+00:00

## timeline

### 18:59:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Never place a just-proven item in the victim slot.** This placement rule has won three times: MRU-end stash demotion (c1), `Back` target promotion (c3), and `Back` probing placement (c5, no regressions, gains on `hm_0`). Where an item lands has mattered more than tuning the admission threshold.\n- **Reuse evidence that survives moves and decays on its own.** The FIFO ghost ring of stash-evicted keys (c5) was the first lever to lift `web_0` 128K. It works because it fixes both counter failure modes: counters that reset on moves and counters that never decay.\n- **The two-hit stash→target gate, with exceptions where it hurts.** It helps at small sizes, and its `web_0` 128K cost has been worn down step by step: a strict room bypass (two free slots, c4), then ghost credit for returning keys (c5).\n- **Plain LRU for stash eviction.** Every attempt to replace it lost hit-rate in all cells (counter-based victim choice c4, LRU-end insertion of fresh puts c2).\n- **Each winner has changed one lever.** The lineage has advanced steadily with small, non-regressing steps built on the previous winner.\n\n## Recurring failures\n\n- **Counters stored in the item.** They stay near zero when they are reset or halved on moves (c2) and pin items permanently when they never decay (c4). Frequency comparison ended up behaving like first-hit promotion. Do not add any counter-based signal without an explicit decay or reset rule.\n- **Changing the target→probing trigger.** Delaying it (c4) and speeding it up (c5) both lost. Each extra crossing sets off an eviction cascade, so the trigger looks near its optimum.\n- **Easier routes into a crowded target at small sizes.** One-hit returns for demoted items, last-free-slot shortcuts, and `Front` insertion of fresh puts all cost hit-rate at 16K. A free slot is not actually free when space is tight.\n- **Wrong diagnoses of `web_0` 128K.** The counter reset on demotion (c3), one-hit churn into probing (c4), and stash flushing (c4) were each ruled out. The real cause was admission of keys returning after stash eviction.\n- **Cost claims are still unverified.** Per-op-time and metadata-size have not been reported in five cycles. Every \"cheaper\" or \"no extra cost\" claim is untested, including the lookup reorder (c1), which leaves hit-rate unchanged, and the ghost ring's metadata.\n\n## Open questions\n\n- Do the ghost ring (c5 v003) and `Back` probing placement (c5 v001) stack? They act on different tiers and different traces.\n- Does a longer ghost reach (a larger factor, or a ring sized to the segment) lift `web_0` 16K, which has stayed flat? Does stale credit then cost `hm_0` 16K?\n- Should demotions from normal buckets record into the ghost, or be checked against it?\n- Is there a trace-adaptive or occupancy-conditioned lever that keeps the `hm_0` gains from protecting residents without hurting `web_0`? The two traces keep pulling timing levers in opposite directions.\n- What do per-op-time and metadata-size look like for the current winner, especially the ghost ring and the target-first lookup reorder?\n\n## Per-policy notes\n\n- **Stash (bucket-local LRU slots):** Use LRU eviction, MRU-end demotion, and the two-hit exit gate. Grant ghost credit to returning keys. Levers applied at this tier act on a small shared population, so they interact rather than stack.\n- **Target/normal tier:** Promotees go to the MRU end (`Back`). The gate is bypassed only when the target has two free slots. With 2-slot buckets that means only when the target is empty, so this lever has little room left.\n- **Probing tier:** Leave the trigger alone. MRU placement for promotees helps `hm_0`, whose hot set reaches probing.\n- **Trace/size cells:** `web_0` 128K has been the persistent bottleneck. When targets have room it favors fast, unfiltered admission, but it responds to history-based admission. `hm_0` rewards protecting residents. 16K cells punish any easier route into the target. `web_0` 16K has been the hardest cell to move since c5.\n"
}
```

### 19:00:01 chat

I wrote `cross-cycle-insights.md` from the five-cycle ledger, using the four required sections in order.

- **Recurring wins:** The rule that has held up most is not putting an item that just proved itself in the slot evicted next. That paid off three separate times. The ghost ring of stash-evicted keys in cycle 5 is the first lever to move `web_0` 128K. The two-hit gate (now with a strict free-slot bypass) and plain LRU for stash eviction have also held up.
- **Recurring failures:**
  - Hit counters stored in the item stayed near zero when reset on moves, and pinned items when they never decayed.
  - Changing when promotees move up to probing lost in both directions.
  - Any easier route into a crowded target cost hit-rate at 16K.
  - Three earlier explanations for the `web_0` 128K weakness were ruled out.
  - Per-op-time and metadata-size have not been measured in any of the five cycles, so every cost claim, including the ghost ring's metadata, is still unverified.
- **Open questions:** whether the ghost ring and the cycle-5 probing placement add up when combined, whether a longer ghost lifts `web_0` 16K, and whether demotions from normal buckets should use the ghost. Also open: how to keep the `hm_0` gains without hurting `web_0`, and actual cost measurements.
- **Per-policy notes:** grouped by stash, target tier, probing tier, and the trace/size cells.


---

- finished: 2026-10-28T19:00:02+00:00
- duration: 22.62s
- status: ok
- input_tokens: 4
- output_tokens: 2070
- cache_read_tokens: 38580
- cache_write_tokens: 10226
- cost_usd: 0.13094
