# Agent log: variant-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/variants/v003
- allow_edits: True
- started: 2026-10-28T14:54:38+00:00

## timeline

### 14:54:43 tool: Bash

```json
{
  "command": "C=/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles; cat $C/0004/hypotheses/H03.md; echo ----; cat $C/0004/variants/v003/eval/fitness-vector.json; echo ----; cat $C/0002/variants/v002/eval/fitness-vector.json; echo ----; cat $C/0002/variants/v002/insight.md; echo ----; ls -R $C/0004/variants/v003 $C/0002/variants/v002/attempts/01/overlay",
  "description": "Read hypothesis, fitness vectors, parent insight, list files"
}
```

### 14:54:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/hypotheses/H03.md"
}
```

### 14:54:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/variants/v003/eval/fitness-vector.json"
}
```

### 14:54:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0002/variants/v002/eval/fitness-vector.json"
}
```

### 14:54:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0002/variants/v002/insight.md"
}
```

### 14:54:51 thinking



### 14:54:51 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/variants/v003"
}
```

### 14:54:53 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/variants/v003/attempts/01/overlay/src/associative_cache/associative_cache_bucket.rs"
}
```

### 14:55:17 thinking



### 14:55:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-28_14-10-09-632/cycles/0004/variants/v003/insight.md",
  "content": "# Variant v003 Insight\n\n## Helped\n\n- hit-rate (`web_0` at 131072, small): A first re-hit now lands at `len/2 - 1` and no longer jumps to MRU. `web_0`'s once-only re-references therefore stop pushing every protected block down one rank. In 1024-slot buckets, where v002's insight blamed pollution of the protected region, this gave back a little of that loss.\n- hit-rate (`hm_0` at 16384, essentially flat): The refutation condition in the hypothesis was not triggered. In 128-slot buckets, one-hit reuse on `hm_0` did not depend on full MRU protection. It did not gain from graded promotion either.\n\n## Hurt\n\n- hit-rate (`hm_0` at 131072, the largest move): In the code, a block below the boundary can reach the upper half only by being hit while it sits exactly in the single slot `len/2 - 1`. Every later probation hit moves the earlier occupant one slot down, back below the boundary, and its next hit only returns it to that slot. So `hm_0`'s long-reuse blocks, whose second hit arrives after many other promotions, almost never get into the protected half. That half is refreshed mostly by the 1/32 BIP samples and gets stale.\n- hit-rate (`web_0` at 16384): In 128-slot buckets, the nearly frozen upper half takes up about half of each bucket. The capacity that actually adapts is roughly the lower half, so the small-bucket config lost more to wasted protected space than it gained from less pollution. This is the opposite of the hypothesis's prediction that `web_0` would improve at both sizes.\n- (`per-op-time` was not measured, even though the hypothesis asked for it. The mid-vector `remove`/`insert` costs no more than the MRU move it replaces.)\n\n## Takeaway\n\n\"Two hits to reach MRU\" did not work as designed here. All lower-half hits share one landing slot, so \"hit again while in the protected region\" became \"hit again before any other probation hit\". In effect the protected half was cut off from the reuse stream. A graded-promotion retry needs a protected landing region that holds position: for example, insert at the protected bottom and let protected items age down normally, or track a per-item hit bit. Shifting a single boundary slot is not enough. Separately, the implementation sends the boundary slot itself (`position == len/2 - 1`) to MRU, while the hypothesis specified `position < len/2`. Under the literal spec that slot would never promote, so even this narrow path into the protected half came from the implementation choice.\n"
}
```

### 14:55:33 thinking



### 14:55:33 chat

I wrote `insight.md` for v003. The change to two-step promotion did not work: overall hit-rate came out slightly below v002.

- **What moved:** `web_0` at 131072 improved a little, and `hm_0` at 16384 stayed flat, so the hypothesis's refutation test wasn't triggered. `hm_0` at 131072 dropped the most, and `web_0` at 16384 also dropped, against the prediction that `web_0` would improve at both sizes.
- **Likely cause (from reading the code, not measured):** every first hit lands on the same single slot, `len/2 - 1`. Each new first hit pushes the previous occupant back below that slot, so a block reaches the top half only if it's hit twice with no other first hit in between. Long-reuse `hm_0` blocks almost never manage that. The top half is then refreshed mostly by the 1-in-32 new items that are inserted straight at the top, and it goes stale.
- **Implementation differs from the spec:** the code sends a hit on the slot at `len/2 - 1` to the top, while the hypothesis said any hit below `len/2` should move to `len/2 - 1`. Under the spec's wording, that slot would never lead to the top at all.
- **Not measured:** `per-op-time`, even though the hypothesis asked for it.

The suggested next step is to give promoted items a landing region that keeps its position, either by inserting at the bottom of the top half and letting items age down normally, or with a per-item hit bit.


---

- finished: 2026-10-28T14:55:34+00:00
- duration: 56.26s
- status: ok
- input_tokens: 12
- output_tokens: 4478
- cache_read_tokens: 130937
- cache_write_tokens: 15662
- cost_usd: 0.241091
