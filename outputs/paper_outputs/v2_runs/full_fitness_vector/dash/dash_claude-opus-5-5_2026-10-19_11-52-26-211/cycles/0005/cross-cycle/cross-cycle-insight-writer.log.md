# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-19T13:51:47+00:00

## timeline

### 13:52:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Stash ordered as a probationary queue by proven reuse (c1–c2).** Demotees enter the stash at the MRU (Back) end and fresh puts enter at the LRU (Front) end. The two gains add up, and the result is the stable admission baseline for every later lineage.\n- **Using idle capacity is the only hit-rate lever since c2 (c4–c5).** The two-choice stash (neighbour stash bucket) and rescuing stash victims into normal-tier holes each raise hit-rate. They draw on different capacity, so their gains stack. Neither adds metadata.\n- **State-neutral lookup reordering moves latency onto rare paths (c1, c5).** A key lives in only one slot, so reordering probes never changes hit-rate. That has been confirmed several times by exact hit-rate matches. Scanning the rarest tier last (currently `s1`) charges only misses and the hits that land there.\n- **Combinations reproduce the expected state (c2, c5).** Independent levers such as lookup order, insert position, neighbour stash and rescue have composed cleanly every time they were tested.\n- **Metadata-size has never moved.** Every change so far derives its decisions from the hash, bucket position or fullness.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge (c2–c3).** MRU placement on stash→target promotion, a second-hit gate, and native-ownership gates on target→probing all lowered hit-rate. The tiny buckets can't hold items long enough for an extra wait to pay off. Front placement on promotion already filters well enough.\n- **Eager filling of holes with unproven items (c5).** Putting fresh items into target holes fills the normal tier with cold entries. Later reuse-proven promotions then trigger eviction cascades. Late filling (rescue) gets the same capacity without that cost.\n- **Pure bookkeeping trims on the promotion path (c4).** Removing redundant hashing or re-scans doesn't reach get p50. The median is set by the scan on the common lookup path and by the moves themselves.\n- **Tweaks to rarely-triggered placement paths (c1).** Balanced placement on stash hits was safe but had almost no effect.\n- **Treating a fixed lookup order as universal (c1→c3).** Target-first won under the old placement, and stash-first won once fresh items were placed at stash Front. Each new placement policy needs its own order test.\n- **Trusting raw latency deltas.** Large shifts on paths a variant never touched appeared repeatedly. Only credit per-op-time changes that have a code-level mechanism and a baseline measured in the same run.\n\n## Open questions\n\n- Does the c5 parent (two-choice stash + rescue, `s1` probed last) keep hit-rate exactly and recover get p50 as predicted? This is not yet verified.\n- Can most `s1` probes on misses be skipped cheaply, for example with a per-`s0` overflow bit or fingerprints? What would that cost in metadata-size?\n- How much does each of the neighbour-stash and rescue levers contribute separately? Would an independent second hash for `s1` beat the fixed `+1` stride?\n- Can the promotion cascade be made cheaper without changing who gets promoted, for example with in-place swaps or a bounded cascade depth?\n- Structural levers not yet tried: stash capacity and associativity, and eviction order inside target buckets. Bigger stash buckets might also let a second-hit filter work where 2-slot buckets couldn't.\n- Is there a form of fresh-into-target fill that always gives way to promoted items and adds hit-rate on top of rescue?\n- Hit-rate and get latency are now traded off directly, because more residency means more scanning. Can cheaper per-bucket scans break that link?\n\n## Per-policy notes\n\n- **Stash tier (holds most capacity):** this is the main hit-rate lever. Insert position matters a lot (reuse-ranked queue), and spreading over a neighbour bucket helps. The stash-hit → promote → demote cascade is the slowest hit path, and it carries more of the load as hit-rate rises.\n- **Target tier:** very small buckets. Promotions in should land at Front/LRU, because MRU placement evicts residents that have already been hit. Free target holes are best filled late by rescued victims, not eagerly by fresh puts.\n- **Probing tier:** gating promotions into probing (ownership or second hit) costs hit-rate. Climbing within the bucket under plain LRU order keeps the tier selective enough.\n- **Lookup order:** the best order depends on the placement policy. On the current lineage it is `s0 → target → probing → s1`. The remaining get gap is the extra bucket that misses scan.\n"
}
```

### 13:52:13 thinking



### 13:52:13 chat

I wrote `cross-cycle-insights.md` in the cycle-0005 cross-cycle directory. It uses the four required sections in order and is built only from the ledger for cycles 1–5.

- **Recurring wins:**
  - **Stash order:** stash entries are ranked by proven reuse. Demoted items go in at the back of the stash and fresh puts at the front.
  - **Spare capacity:** filling empty slots elsewhere is the only thing that has raised hit-rate since cycle 2. The two ways of doing it (a neighbouring stash bucket and rescuing evicted items into normal buckets) stack.
  - **Lookup order:** changing the order buckets are checked in never changes hit-rate. Checking the rarest bucket last moves the cost onto misses.
  - **Combining and metadata:** combined changes reproduced the expected cache state, and metadata-size has never moved.
- **Recurring failures:**
  - **Promotion filters:** extra checks before promoting an item, in either direction, lowered hit-rate.
  - **Eager filling:** putting fresh items straight into empty normal-bucket slots fills them with cold items and makes later promotions more expensive.
  - **Small changes:** trimming bookkeeping didn't reach get p50, and tweaks to rarely used paths barely changed anything.
  - **Fixed lookup order:** the best order depends on how items are placed, so it needs re-testing when placement changes.
  - **Latency numbers:** only latency changes with a clear cause in the code can be trusted.
- **Open questions:**
  - Cycle 5's proposed parent still needs checking.
  - It's unknown whether most checks of the neighbouring stash bucket on a miss can be skipped cheaply, and what that would cost in metadata.
  - It's unknown how much each of the two spare-capacity changes adds on its own.
  - Promotions might be made cheaper without changing which items get promoted.
  - Stash size and target-bucket eviction order haven't been tested.
- **Per-policy notes:** these are grouped by stash tier, target tier, probing tier and lookup order.


---

- finished: 2026-10-19T13:52:14+00:00
- duration: 26.72s
- status: ok
- input_tokens: 4
- output_tokens: 2432
- cache_read_tokens: 37277
- cache_write_tokens: 11574
- cost_usd: 0.148703
