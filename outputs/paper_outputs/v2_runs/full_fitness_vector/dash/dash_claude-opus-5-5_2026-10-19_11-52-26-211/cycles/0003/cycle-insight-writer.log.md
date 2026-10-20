# Agent log: cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003
- allow_edits: True
- started: 2026-10-19T12:17:18+00:00

## timeline

### 12:17:21 tool: Bash

```json
{
  "command": "cd /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003; for f in variants/v00{1,2,3}/insight.md hypotheses/H0{1,2,3}.md; do echo \"=== $f\"; cat $f; done",
  "description": "Read variant insights and hypotheses"
}
```

### 12:17:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v001/insight.md"
}
```

### 12:17:23 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v002/insight.md"
}
```

### 12:17:24 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/variants/v003/insight.md"
}
```

### 12:17:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H01.md"
}
```

### 12:17:25 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H02.md"
}
```

### 12:17:26 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/hypotheses/H03.md"
}
```

### 12:17:28 thinking



### 12:17:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0003/cycle-insight.md",
  "content": "# Cycle 0003 Insight\n\n## Cohort outcome\n\nAll three variants were built on cycle-2's v003 (fresh puts at stash Front, demotees at stash Back). v001 ported the target→probing→stash lookup order. v002 required a second stash hit before promoting to target. v003 blocked target→probing promotions that would evict a native neighbour. All three are listed as Winners, but each one met its own falsification condition. None raised hit-rate. v002 and v003 shortened the get tail but lost a little hit-rate. v001 left hit-rate unchanged, as expected, but made get p50 worse.\n\n## Patterns\n\n- The parent's admission policy now looks like a local optimum on both edges. Cycle 2 showed that hotter stash→target promotion hurts. This cycle showed that stricter promotion on either edge also hurts: a second-hit gate on stash→target, and a native-ownership gate on target→probing. Every filter tried so far has blocked real reuse rather than noise. The tiny buckets (2-slot stash, small target) can't hold items long enough for an extra wait to pay off.\n- How hits are spread across tiers depends on placement policy. Changes that hold for one cache state can stop holding when placement changes. Fresh-at-stash-Front placement sends every key's first reuse through the stash. That made stash hits common enough to flip the lookup-order result: cycle 1 found target-first was a win, and here stash-first is better.\n- The two latency wins share one cause: skipping the remove → promote → demote → stash-insert cascade. Both removed it by promoting less often, and both paid for it in hit-rate. The tail cost comes from how expensive each cascade is, not from how often admission is allowed.\n- The signal used to decide admission matters. Bucket position and native/foreign ownership are free to read. But neither tracks recency well enough to beat plain LRU-ordered promotion here.\n- Metadata-size stayed the same in every variant. Put-path movements had no cause in the code changes and look like noise.\n\n## Open questions\n\n- Can the promotion cascade be made cheaper without changing which items get promoted? Options include in-place swaps instead of remove + reinsert, or limiting how far a cascade can go.\n- Can each bucket scan be made cheaper so get p50 falls under any lookup order? Small per-slot fingerprints would do this, but they would cost metadata-size.\n- Would a lookup order chosen per key or adaptively beat a fixed stash-first order? Or does the extra branching cost more than it saves?\n- Stash capacity and associativity haven't been tested. Bigger stash buckets might let a second-hit filter work where 2-slot buckets couldn't.\n\n## Continuation\n\nKeep cycle-2's v003 as the parent. Nothing this cycle beat it on hit-rate, and v001's lookup order is a regression on this lineage. Drop admission filters on both promotion edges (second-hit, ownership-based) and drop fixed lookup-order changes. Go after get latency through cheaper cascades and cheaper bucket scans that leave cache state unchanged. For hit-rate, try structural levers instead: stash sizing and associativity, or eviction order inside the target buckets.\n"
}
```

### 12:17:48 chat

I wrote `cycle-insight.md` for cycle 0003. All three Winners ran into their own failure conditions, and none of them improved on the parent, cycle-2's v003.

- **v001 (lookup order target → probing → stash):** Hit-rate stayed the same, which shows the change didn't break anything. But the median get time went up. In this setup fresh puts land in the stash, so a key's first re-read is always found there. Checking the stash first is now the faster order, which reverses what cycle 1 found.
- **v002 (promote from stash only on a second hit):** The slowest gets got faster, but hit-rate dropped. The 2-slot stash buckets push items out before their second read arrives.
- **v003 (don't let a promotion evict a neighbour bucket's own key):** The slowest gets also got faster, and hit-rate dropped slightly. The neighbour key being protected is always the least recently used one in its bucket, so whether it belongs to that bucket says nothing useful.

**Cohort-level points:**
- The parent's rules for what gets promoted, in both directions, now look like a local optimum. Cycle 2 showed that promoting items to a hotter spot hurts. This cycle showed that making promotion stricter also hurts.
- Both latency gains came from skipping the chain of moves a promotion triggers (remove, promote, demote, stash insert). They did that by promoting less often, which is where the hit-rate went. The slow tail comes from how expensive each chain of moves is, not from how often promotion is allowed.

**Continuation for the next cycle:**
- Keep cycle-2's v003 as the parent.
- Drop promotion filters and changes to a fixed lookup order.
- To cut get time without changing what's in the cache, make each promotion's chain of moves cheaper or each bucket scan cheaper.
- To raise hit-rate, try stash size and slots per bucket, or the eviction order inside target buckets.


---

- finished: 2026-10-19T12:17:49+00:00
- duration: 30.85s
- status: ok
- input_tokens: 8
- output_tokens: 2646
- cache_read_tokens: 83294
- cache_write_tokens: 14283
- cost_usd: 0.183875
