# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:53:34+00:00

## timeline

### 15:53:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_15-18-23-656/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Policy changes drive hit-rate. Layout and mechanics changes drive speed and size. The two axes stay largely independent. Segmentation gave scan resistance, and that is established.\n- Mechanics changes on the plain LRU were pure wins: u32 links, a lighter hasher, a head-touch early return and move-based eviction (no key clone).\n- Removing unneeded hash work paid off. The fold-free hasher helped put and shrank the struct on the segmented cache.\n- Reference-bit (second-chance) get path: cheaper get and a small scan-resistance gain.\n- A larger protected share gave small, diminishing hit-rate gains.\n\n## Recurring failures\n\n- Shifting promotion/demotion work between operations (lazy demotion, relink-free get) only moves the cost: get gets better while put or the put tail gets worse.\n- Packing the segment bit, or narrowing links, on a segmented policy saves size but not speed. Masking and NIL checks cost latency, and the extra head/tail pair grows the container.\n- Skipping relinks only for head-adjacent nodes added a branch that few hits could repay.\n- Constant-only and position tweaks (protected-share steps, demote position, `put`-update promotion) are exhausted or within noise.\n- Bundling changes (hash fold plus layout) made attribution impossible. Authors cannot read diffs, so attribution is always inferred.\n- Small per-op-time deltas are noise, as shown by constant-only edits that moved latency. Winner status at that scale does not mean improvement.\n- Infrastructure failures (a Maven 429 in cycle 1) can hide a hypothesis without testing it.\n\n## Open questions\n\n- What is the per-op-time noise floor? Would repeat-run measurement separate real deltas from noise?\n- Can the eviction-walk cap be made adaptive, or the choice of which referenced nodes to spare made cheaper, so that put p99 stays low and hit-rate recovers?\n- Is there any hit-rate lever beyond segment-boundary policy, such as an admission filter, ghost-entry history or sizing?\n- Does a smaller protected share help, or has the trend saturated?\n- Can the segment be stored implicitly by list membership, with no masking?\n- Do the hit-rate and latency findings hold on workloads other than the scan-heavy trace? This has been pending since cycle 1.\n\n## Per-policy notes\n\n- Plain LRU: mechanics changes (u32 links, light hasher, touch fast path, clone removal) compose and give speed and size with no hit-rate change.\n- Segmented LRU: hit-rate gain over plain LRU on scans. The remaining cost is promotion/demotion relinking on the hot path, and no work-shifting trick has removed it. The current baseline is the cycle-0006 larger-protected-share variant, with the fold-free hasher and the reference bit.\n- Second-chance (CLOCK-like) eviction: the cap on the walk trades put latency against hit-rate directly. Treat it as a dial.\n- Next candidates: admission filter or ghost history. Evaluate on non-scan traces.\n"
}
```

### 15:53:54 chat

I wrote `cross-cycle-insights.md` to the output dir, with the four fixed sections in order. I regenerated it from the ledger digest alone.

- **Recurring wins:**
  - Policy drives hit-rate; mechanics drive speed and size.
  - Dropping unneeded hash work paid off, as did the reference-bit get path.
  - A larger protected share gives only small, diminishing gains.
- **Recurring failures:**
  - Moving promotion and demotion work between `get` and `put` only relocates the cost.
  - Bit-packing and narrower links on the segmented policy save size but not speed.
  - Constant and position tweaks are within noise.
  - Bundled changes can't be attributed.
  - An infrastructure failure hid a hypothesis in cycle 1.
- **Open questions:**
  - What is the per-op-time noise floor?
  - Can the eviction-walk cap be made adaptive?
  - Is there a hit-rate lever beyond segment-boundary policy?
  - Does anything hold on workloads other than the scan-heavy trace? That check has been pending since cycle 1.
- **Per-policy notes:** these cover plain LRU, segmented LRU, and second-chance eviction. The current baseline is the cycle-0006 larger-protected-share variant, with the fold-free hasher and the reference bit.

I used only qualitative terms, with no invented numbers.


---

- finished: 2026-10-20T15:53:54+00:00
- duration: 20.30s
- status: ok
- input_tokens: 4
- output_tokens: 1644
- cache_read_tokens: 38726
- cache_write_tokens: 10447
- cost_usd: 0.062751
