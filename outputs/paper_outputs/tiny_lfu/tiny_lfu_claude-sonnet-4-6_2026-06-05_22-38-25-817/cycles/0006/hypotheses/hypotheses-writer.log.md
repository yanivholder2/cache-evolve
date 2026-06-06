# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses
- allow_edits: True
- started: 2026-06-06T02:20:29+00:00

## timeline

### 02:20:32 thinking

Let me read all the context files to understand the current state of the research and formulate 3 distinct hypotheses for cycle 0006.

### 02:20:33 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/*/"
}
```

### 02:20:33 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/cycle-winners.json"
}
```

### 02:20:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/cycle-insight.md"
}
```

### 02:20:34 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/leaderboard.md"
}
```

### 02:20:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles-ledger.jsonl"
}
```

### 02:20:35 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cross-cycle-insights.md"
}
```

### 02:20:39 thinking

Now let me read the cycle 0005 variants in detail, and the baseline implementation.

### 02:20:39 chat

Now let me read the variant overlays and the baseline source code.

### 02:20:40 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/*/"
}
```

### 02:20:40 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline/**"
}
```

### 02:20:42 thinking

The baseline directory doesn't have source files listed. Let me look more broadly.

### 02:20:43 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005 -type f | sort"
}
```

### 02:20:43 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/inputs/subjects/tiny_lfu -type f | sort"
}
```

### 02:20:46 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/**"
}
```

### 02:20:48 thinking

Now let me read the actual overlay code from cycle 0005's variants and the baseline to understand the implementation details.

### 02:20:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:20:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v001/insight.md"
}
```

### 02:20:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v002/insight.md"
}
```

### 02:20:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0005/variants/v003/insight.md"
}
```

### 02:26:04 thinking

Now I have a thorough picture of the current state. Let me synthesize what I know:

**Current champion**: cycle/0005/v001 with hit-rate=0.7398 (shown as 0.74 on leaderboard)
- Implementation: 90/10 SLRU, k=4 victim scan, PROMO_THRESHOLD=8, `was_protected` bypass, TinyLFU admission gate

**What's been proven not to work:**
1. Sketch aging (counter halving) — harmful at any cadence
2. Ghost-set re-admission — harmful (evicted items are genuinely cold)
3. Admission-window reservation — too expensive in terms of capacity
4. Split-ratio adjustments beyond 90/10 — already saturated
5. Widening victim scan from k=4 to k=8 — yields no improvement
6. Put-path promotion symmetry — no benefit
7. Ghost-set combined with k=4 scan isolation — ghost-set remains harmful, put-path is a dead end

**What's been suggested but not tried:**
1. **Frequency-ordered eviction within protected segment** — instead of evicting the LRU tail from the protected region, evict the least-frequently-used item there
2. **TinyLFU sketch tuning** — adjusting sketch size, frequency threshold for admission, or counter behavior

The cycle analysis points toward frequency-aware ordering within the protected segment as the next pivot — evicting the coldest protected item by frequency rather than recency. If that doesn't work, the admission gate and sketch configuration are the remaining structural levers to pull. I'm laying out three distinct hypotheses to test: frequency-ordered eviction in the protected segment is the most recommended unexplored direction, and TinyLFU sketch admission gate tuning is the second axis worth investigating.

Looking at the sketch configuration itself, the current setup uses a depth of 4 and width factor of 4x capacity with a minimum width of 16. Increasing the width factor would reduce hash collisions and improve frequency estimate accuracy, which could sharpen both the k=4 scan decisions and the promotion threshold logic. That's a cleaner lever than tweaking the admission inequality. The key structural improvements would be frequency-ordered eviction within protected—scanning the first k items and demoting the minimum-frequency one instead of always evicting the LRU head—and relaxing the admission gate to `>=` so items that tie the victim's frequency get a chance to enter. I'm also considering doubling the sketch width factor for better frequency estimation accuracy. **H02** doubles the sketch array size to reduce hash collisions and sharpen frequency discrimination for both the k=4 scan and admission threshold. **H03** relaxes the admission gate from strict inequality to inclusive, letting newcomers that match the victim's frequency enter the cache. I was reconsidering the tie-breaking logic for H03—

actually, I need to ensure at least one hypothesis uses `cycle/0005/v001` as parent per the requirement, and using that incumbent for both H01 and H02 gives them a shared implementation base, which is acceptable for exploring different axes from the same starting point.

For H03, I could branch off with a completely different structural approach—maybe a decay mechanism for protected items or a size-aware strategy that hasn't been tested yet. I'm considering several unexplored directions: adaptive protected capacity that adjusts based on miss rates, frequency decay within the protected segment to age out stale items, two-choice eviction for better selection, or an absolute frequency threshold for admission rather than relative comparison.

Let me lock in three concrete hypotheses. H01 would apply frequency-sampled demotion to the protected segment—scanning the first few LRU items and demoting the one with minimum sketch frequency instead of blindly demoting the coldest. This mirrors the k-way victim selection already used in probationary, but applied to the protected→probationary path, preserving high-frequency items that happen to be near the LRU tail. I'm considering doubling the sketch width factor from 4 to 8, which would reduce hash collisions and improve frequency estimation accuracy across the board—sharpening the victim selection in probationary, the promotion threshold, and admission decisions. This parameter hasn't been directly tuned before, so there's real potential here.

But I'm skeptical whether it would actually move the needle on hit rate. At the current width of 8192 entries per row with depth 4, the error bounds are already reasonable for a 2048-item cache, so this might be a null result.

I'm also exploring tweaks to the admission gate logic—either making it more conservative by requiring newcomers to be strictly hotter than victims, or more lenient by giving them a slight edge. And there's something interesting about how we initialize frequencies for new items that I want to dig into. The ghost-set approach for admission doesn't work because those evicted items are genuinely cold and shouldn't be fast-tracked back in. Instead, I'm considering two concrete improvements: one that applies frequency-sampled demotion to protected items (picking the coldest one to demote rather than always the LRU head), and another that tightens the admission gate by requiring incoming frequency to exceed victim frequency by some margin rather than just matching it.

The stricter admission gate could backfire on Zipfian distributions by rejecting legitimately hot items early on, but it might reduce churn from borderline cases. There's also a third angle: rethinking how the `was_protected` flag works — instead of always re-promoting protected items on next access, I could check if their frequency meets a lower threshold (like half the normal promotion threshold) to avoid reflexively restoring items that are actually cold now.

Since the cycle framework treats the five knobs as fixed, I should focus on structural changes I haven't explored yet, particularly frequency-ordered eviction within the protected segment and sketch/admission tuning. Let me outline a new variant where protected overflow uses frequency sampling rather than simple LRU demotion.

**H01** scans a small window of items from the LRU tail of protected and demotes whichever has the lowest sketch frequency, preserving high-frequency items that happen to be old. **H02** doubles the sketch width to reduce hash collisions and sharpen frequency discrimination across all the gates that depend on it. **H03** is the half-threshold bypass I was considering earlier.

Actually, reconsidering H03 — the cycle insight showed that the `was_protected` bypass tends to unconditionally re-promote most items anyway, so maybe the real issue is that I'm not being selective enough about which demoted items deserve a second chance.

Let me explore a different angle: what if I only refresh a protected item's position in the LRU order when it's accessed with high frequency? Items accessed once stay in place, letting low-frequency items naturally drift toward the LRU head for demotion, while genuinely hot items bubble to the MRU end. But this creates a problem — items that are legitimately popular but accessed infrequently relative to others would get unfairly evicted.

A simpler approach: I could modify the admission gate to use `>=` instead of `>` when comparing incoming frequency to victim frequency. This means when a new item ties the victim's sketch frequency, it gets admitted rather than rejected. On Zipfian traces, these ties happen frequently enough to matter.

But I need to be careful here—cycle 0003 already explored a promotion threshold, which is different from the admission gate. Looking at the actual code, the admission check is strict inequality. Changing it to inclusive would let newly-heated items with frequency=1 compete against existing frequency=1 items in the cache. The tradeoff is that this increases churn among equal-frequency items, which could help by keeping fresher items or hurt by creating unnecessary evictions.

Actually, thinking through the Zipfian distribution more carefully: the hot items are already protected with high frequencies, and the probationary tail is full of cold items at frequency 1-3. A pre-heated newcomer at frequency=1 would tie with these victims, so admitting on ties doesn't move the needle much.

Let me explore something more promising for H03—a graduated promotion threshold that adapts based on how long an item has been sitting in probationary. Instead of a fixed threshold, I'd track per-item age and use a lower threshold for items that have been waiting longer.

The simplest approach: count how many times an item gets accessed while in probationary, and if it hits 3 accesses without reaching the full promotion threshold, promote it anyway. This would require adding a new field to track access count in the Node struct.

But there's a tension here — if most items get promoted by this access-count gate before hitting the sketch-frequency threshold, then the original PROMO_THRESHOLD becomes mostly irrelevant. On a Zipfian distribution, hot items would accumulate accesses quickly and get promoted through this path instead.

The real distinction is that PROMO_THRESHOLD counts global sketch frequency (including accesses before the item entered the cache), while the probationary-access-count gate only tracks accesses *while* the item is in probationary. So this gate would catch moderately popular items that are accessed 3 times in probationary but haven't yet accumulated enough total accesses to cross the sketch threshold — though I'm not sure if that's actually a meaningful difference or just redundant with lowering PROMO_THRESHOLD.

Let me step back and focus on concrete hypotheses to test: frequency-sampled demotion from protected seems like the most promising unexplored direction, then there's widening the sketch for better frequency accuracy, and I should also explore something structurally different from those two approaches.

For that third hypothesis, I'm considering tuning the TinyLFU admission gate itself—instead of just checking if the incoming item's frequency exceeds the victim's, I could add a global minimum admission threshold that prevents very cold items from ever entering the cache, even if they match a cold victim's frequency. This would be more selective about what gets admitted in the first place.

But I'm realizing this might be too conservative and could hurt hit rate by rejecting items that could still be useful. Let me think about what genuinely hasn't been explored yet—maybe something like applying frequency decay within the protected segment's LRU ordering, so that when items overflow from protected, we're not just picking by recency but also considering how cold they've become relative to their historical frequency.

Actually, that's essentially what H01 already does with its frequency scan. So let me consider a different approach: what if instead of promoting probationary items immediately when they hit the promotion threshold, I defer that promotion until the protected segment has some breathing room? If protected is full and an incoming item's frequency is below a certain level, I could keep it in probationary even if it technically qualifies, only promoting when either the frequency is high enough to justify the disruption or when protected has capacity. Though that's getting complex—it's basically raising the promotion threshold dynamically based on protected's fullness.

Another angle entirely: adding a second frequency counter specifically for the protected segment, separate from the global sketch, to get more granular frequency tracking for items that matter most. The key difference here is that this per-node counter only tracks accesses while the item is in protected, whereas the global sketch counts all accesses everywhere—so an item that was popular before entering protected but hasn't been touched since would have high sketch frequency but low `prot_accesses`, making this approach potentially more accurate for deciding what to demote. I'm formalizing this as H03: adding a `prot_accesses` field to each node that increments on every access while in protected, resets when demoted, and then during demotion I scan k=4 items from the protected head to find the one with the lowest counter.

But I'm realizing this is pretty similar in spirit to H01—both are frequency-sampled demotion from protected, just using different frequency signals. Having two hypotheses target the same mechanism feels redundant. Let me pivot H03 to something genuinely different instead, maybe targeting the CountMinSketch hash function itself—using a different mixing strategy like XOR with a fixed constant to reduce row correlation and improve accuracy. It's a technical tweak that probably won't move the needle much, but it's a distinct angle.

Looking at what the cross-cycle analysis flagged as remaining opportunities: frequency-ordered eviction within protected (H01), TinyLFU admission gate tuning like sketch size and frequency thresholds (H02 covers sketch size but not threshold or counter initialization), and cache-size sensitivity. For counter initialization specifically, what if we give items a frequency boost when they're first inserted—essentially a doorkeeper pattern using a Bloom filter to only count items seen more than once?

The academic TinyLFU paper actually describes this mechanism: a **DoorKeeper** Bloom filter that sits in front of the CountMinSketch. On first access, items get recorded in the Bloom filter but skip the sketch increment. On second access, they're already in the DoorKeeper, so they proceed to the sketch. This filters out one-hit-wonders and reduces noise in the frequency estimates. I'm proposing this as H03—a structural change that's distinct from everything we've tested so far. The tricky part is handling the DoorKeeper's lifecycle: if we never reset it, one-hit-wonders from early in the cache's lifetime will permanently block new items from being sketched. For H03, I'm considering changes to the admission gate logic in `put()` — specifically using a secondary tie-breaking criterion when incoming and victim frequencies are equal. The idea is to prefer admitting the incoming item if the victim never had protected history, or to implement a frequency-relative admission gate that adjusts the comparison threshold based on some property of the items being compared. we haven't directly explored yet. The idea is to require incoming items to meet an absolute frequency threshold—say, at least 3 accesses—before they're even eligible for admission into the cache, separate from the victim comparison. This would be a new control lever distinct from PROMO_THRESHOLD, which only gates promotion within the cache structure itself.

So now I have three candidate improvements to evaluate: frequency-sampled demotion from protected, a wider sketch, and this absolute admission floor. Let me think through what the cycle data actually suggested about which direction to pursue. I'm exploring a "lazy frequency-ordered LRU" approach where protected items only refresh to the MRU position if their frequency exceeds a threshold, letting cold items naturally drift toward demotion. This simplifies the logic by using a constant threshold instead of computing averages, though it does change how protected items behave—those below threshold won't move to MRU on access and could get demoted sooner. On a Zipfian distribution, items in the protected segment that fall below the frequency threshold are likely stragglers or early promotions that haven't been accessed recently—refreshing their LRU position would be wasteful, so skipping the refresh lets them age out naturally. This targets a different mechanism than the demotion scan in H01, making it a structurally distinct hypothesis.

But I'm realizing H02's parent choice matters: if the parent is baseline, the implementer only adds the sketch width change without the promotion threshold, demotion scan, or protected tracking that H01 introduced—that could create unexpected interactions and miss the synergies those features provide. Testing in isolation on baseline would reveal whether sketch width has a standalone benefit, but given that each change in cycle/0005/v001 proved individually valuable, stripping them away would likely hurt more than help. I'll go with cycle/0005/v001 as parent for H01 and H03, and use baseline for H02 to understand sketch width's effect without the accumulated complexity. - H03: baseline (testing a fundamentally different design approach from scratch)

For H03, I'm considering Window TinyLFU, which uses a small admission window (around 1% of capacity) as a recency buffer in front of the main SLRU. This differs from the earlier admission window attempt because that one had too large a window; W-TinyLFU keeps it minimal—maybe 20 items for a 2048-capacity cache. Items only graduate from the window to the main cache if they outperform the SLRU eviction candidate, which is a well-established technique from the original W-TinyLFU paper.

The implementation would involve a tiny window segment (1-2% of capacity), a main SLRU with a 90/10 split, and comparison logic when the window overflows. It's more complex structurally, but genuinely distinct from what we've tried. The concern is whether a tiny window helps much on this Zipfian trace given that the TinyLFU sketch already handles one-hit-wonders effectively.

Looking back at cycle 0001's v002 admission window attempt, the problem was capacity cannibalization—reserving a permanent window segment shrunk the main LRU, and that structural loss showed up immediately before any benefits could materialize.

I've been going in circles here. Let me lock in concrete hypotheses instead. For H01, I'm planning a frequency-sampled demotion scan within the protected segment—instead of always demoting the head of protected, I'll scan a few items from the LRU end and demote whichever has the lowest sketch frequency, using the same scan logic we apply to probationary victims but for the protected-to-probationary demotion path.

H02 is simpler: just widen the CountMinSketch by doubling the width factor from 4 to 8, which gives more accurate frequency estimates across the board without touching any other logic.

For H03, I'm starting to think a symmetric frequency-sampled victim selection with tie-breaking by sketch depth might be overcomplicating things—let me ground this back in something simpler rooted in the baseline.

One angle is improving the hash diversity in the CountMinSketch itself. The current implementation uses DefaultHasher with seeds 0, 1, 2, 3, but if those seeds have correlations, the min-estimate gets systematically biased. Using more diverse seeds like prime constants would reduce those correlated errors, though the practical impact is probably small.

Another baseline-inspired approach is conservative update for the sketch: instead of incrementing all depth rows for a key, only increment a row if its current count is already at the minimum. This is a well-known CMS improvement that reduces overcounting and should give more accurate frequency estimates with just a few lines of change in the increment function. The hypothesis is that better estimates would improve the admission gate decisions and raise hit rate.

But there's a catch—if I'm using the baseline as the parent, it doesn't have the k=4 victim scan or PROMO_THRESHOLD logic. So conservative update would only help the admission gate comparison in put(), and the gain might be minimal without those other optimizations.

I'm settling on rooting all three hypotheses from the champion version to maximize the chance of meaningful improvements. H02 and H03 are both targeting sketch accuracy though—one through conservative increments and one through wider sketch width—which feels like near-duplicate work. I need to reconsider whether both are worth pursuing.

Actually, they're attacking the problem from different angles: conservative updates eliminate overcounting from the standard CMS increment pattern, while a wider sketch reduces hash collisions. The gains could be complementary rather than redundant. But to better understand what's actually driving improvements, I should replace H03 with something orthogonal—maybe a conditional refresh strategy in the protected segment that only moves items to the MRU tail when their sketch frequency crosses a certain threshold.

This would be purely about recency management rather than eviction policy or sketch accuracy. But I'm already seeing a potential issue: if a low-frequency item stays in protected without being refreshed, it ages toward the LRU head and gets demoted faster than intended. If it keeps getting accessed but stays below the refresh threshold, it could churn between segments inefficiently.

The real problem is that sketch frequency is global—it includes all accesses before the item even entered the cache. An item promoted from probationary might have high sketch frequency from its history, so a fixed threshold like 4 won't reliably identify items that are actually cold while in protected.

Let me try a different approach: track in-protected hits separately. I'd add a counter to each node that increments only when the item is accessed while in protected, reset to zero on demotion. Then in the demotion selection logic, I'd scan a few items from the LRU end and pick the one with the lowest in-protected hit count.

But wait—this is basically the same mechanism as H01, just using a per-node counter instead of the global sketch. They're both changing how we select demotion victims from protected. I need H03 to be genuinely distinct from H01. Let me think about what else could vary: admission into the cache itself, or the promotion logic from probationary to protected.

For H03, I could target the probationary eviction scan differently—maybe scanning the entire probationary segment instead of just the LRU tail, or using random sampling. But the insight already suggests that the coldest items cluster at the LRU end, so random sampling would mostly just waste effort.

I'm also considering whether scanning from the MRU end instead would help, but that would evict the freshest items, which is clearly worse. A two-list probationary structure could work but adds complexity.

Another approach: what if I initialize the sketch counters to a small positive value instead of zero? This "initial frequency boost" could reduce cold-start bias for new items without history.

But wait — if everything starts at 1, then the PROMO_THRESHOLD gate breaks down since the admission condition `incoming_freq > victim_freq` would fail when both are 1. This would actually hurt new item admission, so that's counterproductive.

I'm also wondering if reducing the sketch width factor to 2 would help by increasing collisions and making the sketch noisier, but that seems unlikely to improve decision quality. And I'm exploring whether using multiple hash seeds could provide some benefit.

Stepping back though, the literature suggests some mechanisms that could genuinely help with Zipfian workloads: frequency-aware size weighting (though this cache assumes uniform item sizes), time-decay for frequency counts (which was already attempted and failed), and temporal locality awareness that LRU already captures.

I'm considering whether a segmented sketch approach could work better — maintaining separate frequency sketches for the probationary and protected segments to get more accurate estimates for each. There's also the possibility of ARC-inspired adaptive segment sizing, dynamically adjusting the boundary based on whether promotions are making good decisions, but that adds significant complexity.

The clearest unexplored direction seems to be changing the admission gate itself. I'm thinking about H03: adding an absolute minimum frequency threshold for admission, so only items that have been seen at least twice can enter the cache, preventing single-occurrence items from displacing anything regardless of the victim's frequency.

The key insight is that `put()` increments the sketch first, so a brand-new key gets frequency 1 after that increment, while a key that's been accessed once before and then put would have frequency 2. With MIN_ADMISSION_FREQ=2, I'd filter out genuine one-hit-wonders that shouldn't be taking up space during cold starts or workload shifts, letting TinyLFU focus on what it does well—handling the actually frequent items. The impact is probably neutral or slightly positive—it's straightforward and testable. Rather than keep spiraling, I should settle on three concrete hypotheses: frequency-guided demotion from protected instead of pure LRU, a conservative CMS update that only increments at minimum count to reduce overcounting, and testing a wider sketch from baseline to isolate its effect. Though I'm noticing both the second and third hypotheses are really about improving sketch accuracy. H03 adds an absolute minimum frequency threshold to the admission gate, so newcomers that have only appeared once get rejected even if they'd displace an equally cold victim, cutting down on low-value churn. These three hypotheses target different parts of the system—H01 tackles demotion within the protected segment, H02 improves sketch accuracy, and H03 gates admission—and they cover the remaining frontiers well while building on what's already working.

Now I'm writing out the concrete implementation details for each hypothesis, starting with H01 which replaces the simple LRU-head demotion with a k=4 scan through the protected segment's tail, estimating frequency for each candidate and demoting the coldest one while breaking ties by age.

For H02, I'm implementing conservative update semantics in the CountMinSketch by first computing the current minimum estimate, then only incrementing the rows where the count equals that minimum, which should reduce overcounting and sharpen frequency signals for both victim selection and promotion thresholds.

For H03, I'm adding a minimum admission frequency constant and updating the put logic to require incoming items to have at least 2 prior sketch hits before they can displace a probationary victim, which should reduce churn from one-time items and improve stability on Zipfian workloads.

Now let me verify these three hypotheses haven't been tried before: H01 applies the k=4 scan idea to protected-to-probationary demotion rather than just victim selection, H02 is a new conservative CMS update approach, and H03 is the first attempt at an absolute frequency floor for admission. All three are novel.

I should set up the output directory before writing these out.

### 02:26:04 chat

Now I have all the context I need. Let me check if the output directory exists before writing.

### 02:26:05 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/**"
}
```

### 02:26:08 thinking

The output directory exists (there's a task.md and hypotheses-writer.log.md). Now let me also check the output directory path more explicitly.

### 02:26:08 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/**"
}
```

### 02:26:13 thinking

OK, so the hypotheses directory has task.md and hypotheses-writer.log.md. Let me now write the three hypothesis files.

Let me also read the v002 and v003 overlays for context, to make sure I fully understand the current champion's code (it's the v001 overlay I already read).

Now let me write the three hypothesis files:

H01: Frequency-sampled demotion from protected (k=4 scan in maybe_demote_from_protected)
H02: Conservative CountMinSketch update 
H03: Absolute minimum frequency floor for admission

Writing hypothesis files...

### 02:26:16 chat

I now have comprehensive context. Let me synthesize the research state and write the three hypotheses.

**What's been proven to work (accumulated in cycle/0005/v001):**
- 90/10 SLRU split
- k=4 frequency-sampled victim scan in probationary eviction
- PROMO_THRESHOLD=8 with `was_protected` bypass

**What's been ruled out:** sketch aging, ghost-set, admission windows, k=8 scan, 95/5 split, put-path promotion, scan widening

**Remaining frontiers identified:** (1) frequency-ordered eviction within protected, (2) TinyLFU sketch/admission tuning — both completely unexplored.

Three distinct hypotheses, each targeting a different code path:

### 02:26:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H01.md",
  "content": "# H01\n\nScanning k=4 candidates from the LRU end of the protected segment and demoting the minimum-frequency one, rather than always the LRU head, reduces protected-segment pollution by cold overflow survivors and improves hit rate.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nModify `maybe_demote_from_protected()` to use a k=4 frequency scan before demoting, mirroring the probationary victim scan that was added in cycle 0004.\n\nCurrent behaviour: the method unconditionally demotes `prot_head` (the single LRU-tail item of protected).\n\nNew behaviour: walk up to `EVICTION_SCAN_K` (4) items starting from `prot_head`, calling `self.sketch.estimate(&self.nodes[idx].key)` on each. Track the index with the minimum estimated frequency (`demotion_idx`), breaking ties by taking the item closest to `prot_head` (i.e., the LRU-oldest in the window). After the scan, demote `demotion_idx` — not necessarily `prot_head` — by calling `prot_unlink(demotion_idx)`, decrementing `protected_len`, setting `self.nodes[demotion_idx].was_protected = true`, calling `prob_push_back(demotion_idx)`, and incrementing `probationary_len`.\n\nThe outer `while self.protected_len > self.protected_capacity` loop is unchanged; only the single-demotion inner body changes.\n\nNo other functions, constants, or struct fields require modification. `EVICTION_SCAN_K` is already defined as 4 and should be reused rather than introducing a new constant.\n\n## Rationale\n\nFive cycles have refined the probationary side to near-optimum, but the protected segment has always used pure LRU ordering internally. When `was_protected` overflow-demotees re-enter protected via the bypass, they land at the MRU tail regardless of their sketch frequency. Over time some cold-ish items accumulate near the LRU head of protected not because they are truly hot but because they have not been accessed recently. A k=4 scan finds the coldest among these without requiring a full-segment traversal, the same trade-off that proved worthwhile for probationary eviction.\n"
}
```

### 02:26:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H02.md",
  "content": "# H02\n\nReplacing standard CountMinSketch increment semantics with conservative update — incrementing only the sketch rows that currently hold the minimum count for a key — reduces systematic overcounting and sharpens the frequency estimates used by both the k=4 victim scan and the PROMO_THRESHOLD gate, improving hit rate.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nModify `CountMinSketch::increment()` to use conservative update semantics. No other functions or constants change.\n\nCurrent implementation increments `self.counts[row][pos]` for every row unconditionally (subject to the `MAX_COUNT` cap).\n\nNew implementation:\n1. Before the per-row loop, compute the current minimum estimate: `let current_min = self.estimate(key);`.\n2. Inside the loop, only increment `self.counts[row][pos]` if `self.counts[row][pos] <= current_min` (i.e., this row is at the minimum; incrementing it will not push it above the true minimum). The `MAX_COUNT` guard remains unchanged.\n\nThe function signature, return type, and all callers remain the same. `estimate()` is unchanged.\n\nConcretely, replace the body of `increment()` with:\n\n```rust\nfn increment<K: Hash>(&mut self, key: &K) {\n    let current_min = self.estimate(key);\n    for row in 0..self.depth {\n        let pos = self.position(key, row as u64);\n        let counter = &mut self.counts[row][pos];\n        if *counter <= current_min && *counter < MAX_COUNT {\n            *counter += 1;\n        }\n    }\n}\n```\n\n## Rationale\n\nStandard CMS increment blindly raises all rows, causing over-estimation whenever two keys collide in the same bucket. Conservative update lifts only the rows that are at the current minimum, tightening the estimate without requiring a wider sketch. This is a well-known improvement to CMS accuracy with no memory cost. Given that hit rate at the current optimum is determined by the fidelity of frequency discrimination in the k=4 scan and the PROMO_THRESHOLD gate, sharpening estimates without changing any other knob is a clean test of whether sketch noise is still a limiting factor.\n"
}
```

### 02:26:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/H03.md",
  "content": "# H03\n\nRequiring incoming items to have accumulated at least 2 sketch hits before they can displace a probationary victim — a hard admission floor — reduces cache churn from first-encounter items and improves hit rate on a stable Zipfian trace.\n\n## Parent\n\ncycle/0005/v001\n\n## Changes\n\nAdd a new constant and tighten the admission gate condition in `put()`.\n\n**Step 1 — add constant** (place near the other constants at the top of `mod.rs`, after `PROMO_THRESHOLD`):\n\n```rust\n/// Absolute minimum sketch frequency an incoming item must reach before it\n/// can displace a probationary victim via the eviction-path admission gate.\n/// Guards against first-encounter items (freq == 1 after the put() increment)\n/// displacing incumbents purely by chance.\nconst MIN_ADMISSION_FREQ: u32 = 2;\n```\n\n**Step 2 — modify the admission gate in `put()`**.\n\nCurrent condition (approximately line 273 in the full-arena branch):\n```rust\nif incoming_freq > victim_freq {\n```\n\nNew condition:\n```rust\nif incoming_freq > victim_freq && incoming_freq >= MIN_ADMISSION_FREQ {\n```\n\nNo other code changes. The `else` branch (reject newcomer) is unchanged. The victim selection scan, the recycle-slot logic, and all other paths are unchanged.\n\n**Effect:** an item calling `put()` for the very first time — with no prior `get()` misses to accumulate frequency — will have `incoming_freq == 1` after the increment at the top of `put()`. This item can no longer displace any probationary resident regardless of the victim's frequency. An item that was seen at least once via a `get()` miss before being `put()` will have `incoming_freq >= 2` and is subject only to the existing `incoming_freq > victim_freq` comparison as before.\n\n## Rationale\n\nThe k=4 victim scan already ensures the evicted item is the coldest probationary candidate, but the admission gate only asks \"is the newcomer hotter than the victim?\" For a cold victim with frequency 1 and a brand-new item also with frequency 1 (after its first put() increment), the gate correctly rejects (strict `>`). However, a truly first-seen item with frequency 1 could in principle beat a victim at frequency 0, which can occur early in trace replay. More importantly, raising the floor to 2 establishes a clean invariant: only items with at least one prior encounter beyond the insert call itself can enter the cache via eviction — exactly the principle the TinyLFU admission gate is designed to enforce. Five cycles have not tested this axis at all; the existing gate has never had an absolute floor.\n"
}
```

### 02:27:07 chat

Three hypotheses written to `/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/hypotheses/`:

---

**H01 — Frequency-sampled demotion from protected** (`cycle/0005/v001`)  
The most-recommended unexplored axis from the cycle-insight. Modifies `maybe_demote_from_protected()` to walk the k=4 LRU-end items of the protected segment and demote the lowest-frequency one (same scan logic already proven for probationary eviction) rather than always taking `prot_head`. Targets the protected segment's internal ordering, which has been pure-LRU across all five cycles.

**H02 — Conservative CountMinSketch update** (`cycle/0005/v001`)  
Changes `CountMinSketch::increment()` to only raise rows that currently hold the minimum count for a key (conservative update), leaving higher rows untouched. This eliminates the systematic overcounting of standard CMS with no memory cost and no logic changes elsewhere. Sharpens the frequency signal used by the k=4 probationary scan, the PROMO_THRESHOLD gate, and the admission comparison — a clean probe of whether residual sketch noise is still limiting hit rate.

**H03 — Absolute admission frequency floor** (`cycle/0005/v001`)  
Adds `MIN_ADMISSION_FREQ = 2` and changes the `put()` admission gate from `incoming_freq > victim_freq` to `incoming_freq > victim_freq && incoming_freq >= MIN_ADMISSION_FREQ`. First-encounter items (freq=1 after the single `put()` increment, never seen via a prior get-miss) cannot displace any resident. Tests the admission-gate tuning axis — the only remaining structural frontier the cross-cycle insight identified — with a minimal, reversible change.


---

- finished: 2026-06-06T02:27:08+00:00
- duration: 398.94s
- status: ok
- input_tokens: 12
- output_tokens: 20078
- cache_read_tokens: 345664
- cache_write_tokens: 38705
- cost_usd: 0.551478
