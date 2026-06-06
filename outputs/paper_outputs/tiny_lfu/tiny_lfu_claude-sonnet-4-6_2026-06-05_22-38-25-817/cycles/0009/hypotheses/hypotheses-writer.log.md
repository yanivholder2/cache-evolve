# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-06-06T02:54:46+00:00

## timeline

### 02:54:49 thinking

Let me read all the relevant context files to understand what has been tried so far and what strategies might work for cycle 0009.

### 02:54:50 tool: Bash

```json
{
  "command": "ls /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/",
  "description": "List cycle 0008 variants"
}
```

### 02:54:50 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/cycle-winners.json"
}
```

### 02:54:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/cycle-insight.md"
}
```

### 02:54:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/leaderboard.md"
}
```

### 02:54:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cross-cycle-insights.md"
}
```

### 02:54:55 thinking

Now let me read the cycle 0008 variants' overlays and fitness vectors to understand what was tried.

### 02:54:56 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v*/eval/fitness-vector.json"
}
```

### 02:54:56 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v*/insight.md"
}
```

### 02:54:59 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/**"
}
```

### 02:54:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles-ledger.jsonl"
}
```

### 02:55:01 thinking

Now let me read the cycle 0008 variant overlays to understand the current state of the code:

### 02:55:02 chat

Now let me read the current winning variant's source code and the baseline:

### 02:55:03 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:55:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v001/insight.md"
}
```

### 02:55:04 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v002/insight.md"
}
```

### 02:55:05 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0008/variants/v003/insight.md"
}
```

### 02:55:09 thinking

Now I have a very complete picture of the state of the system. Let me read the baseline as well to confirm its structure.

Let me also check what the cycle 0007/v001 and cycle 0005/v001 overlay looks like, since cycle 0007/v001 is the cross-cycle winner:

### 02:55:10 chat

Now let me also check the cycle 0007/v001 overlay (the cross-cycle winner) and the baseline:

### 02:55:12 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/attempts/*/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:55:12 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline/**"
}
```

### 02:55:14 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/**"
}
```

### 02:55:18 thinking

There's no overlay directory with the actual source file. That means the variant used the latest attempt's overlay. Let me check under what path the overlay source file is for cycle 0007/v001.

Wait, I see there's `attempts/01/overlay/src/tiny_lfu/mod.rs` but the Glob didn't match it as a result pattern. Actually it's there - I can see it in the task files. Let me read it.

### 02:55:18 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:59:50 thinking

Now I have a very complete picture. Let me analyze the state of the system:

**Current best variant** (cycle/0007/v001 and cycle/0008/v001, both at 0.7435):
- SKETCH_DEPTH: 4 (cycle/0007/v001) or 6 (cycle/0008/v001)
- SKETCH_WIDTH_FACTOR: 8 (8× capacity)
- Conservative update semantics
- EVICTION_SCAN_K: 4
- PROMO_THRESHOLD: 8
- 90/10 SLRU split
- Several axes have been thoroughly tested and ruled out: sketch aging, ghost-set re-admission, SLRU split variations, protected-segment victim scanning, sketch depth differences, hash seed diversity, admission-floor raising, direct-to-protected bypass, and symmetric promotion on puts. The key remaining questions involve revisiting some of the policy knobs that were closed earlier in cycle 0005.

- I've confirmed several wins across the optimization cycles: the conservative CMS update, 8× sketch width expansion, k=4 victim scanning, the 90/10 SLRU split, and the PROMO_THRESHOLD=8 configuration with the was_protected bypass. For cycle 0009, the main recommendation is to re-probe PROMO_THRESHOLD gating and the k=4 victim-scan window now that the sketch signal is sharper and less noisy than when those earlier conclusions were reached.

- There's also a question about whether frequency-ordered eviction within the protected segment still has value—it was tested once in cycle 0006 and showed flat results, but the sharper sketch estimates might reveal different behavior. The PROMO_THRESHOLD=8 was originally calibrated during cycle 0005 when collision noise was higher, before the conservative update and 8× width improvements. With the reduced count inflation from collisions now, the effective frequency distribution of probationary items may have shifted, potentially changing how true hot items accumulate.

The cycle 0005 data showed that lowering the threshold to 4 only promoted second-tier churn candidates, and cycle 0007/v002 confirmed that dropping PROMO_THRESHOLD from 8 to 4 caused a regression. But the cross-cycle analysis flags PROMO_THRESHOLD gating and the victim-scan window as the highest-priority experiments to re-probe against the cleaner baseline, since both were locked down when collision noise was substantially higher. was already tested before the conservative-update and wide-sketch improvements came in, so it might behave differently now. The per-policy notes suggest k=4 is already saturated and widening wouldn't help, but the cross-cycle insight flags re-probing these knobs against the new baseline as a priority. Hmm, but the per-policy notes suggest k=4 is already saturated, so maybe that understanding is already baked in. Still, I wonder if smaller values like k=2 or k=3 could work better—though reducing k would mean less frequency protection overall, which seems counterintuitive. Let me reconsider PROMO_THRESHOLD from a different angle: instead of tweaking the threshold value itself, what if I change when it applies? I'm thinking about approaches like using a frequency-decay-based soft admission floor that adapts based on recency rather than staying fixed at a single value.

There's also the idea of making protection size-aware, though that's already tracked through probationary_len. But what's more promising is frequency-ordered insertion into probationary—instead of always placing new items at the MRU end, I could position them based on their frequency so hot items get a head start and stay protected longer while cold items get pushed toward the LRU end faster. This could be worth exploring, especially for re-admitted items that come back with higher frequency. probationary into warm and cold zones, where eviction always targets the cold section. But I need to focus on what's actually changeable here—just the overlay file in the TinyLFU module, with no modifications to tests or dependencies. Let me narrow down to concrete, testable hypotheses. The first one is lowering the promotion threshold now that the sketch is more accurate with the wider width and conservative updates. With a sharper frequency estimate, items that genuinely have a frequency of 5-7 should now be correctly identified instead of underestimated.

I'm reconsidering the threshold value—cycle 0007 v002 already tested dropping it to 4 and saw a regression, but we haven't explored intermediate values like 6. Given that the current baseline has an even wider sketch at 8× width plus conservative updates and increased depth, the estimates might be precise enough that 6 could be the optimal threshold.

But looking back at the policy notes, this direction seems closed—threshold 2 was neutral and threshold 4 regressed, so there's limited reason to expect threshold 6 to help. The core issue is that lowering the gate just promotes second-tier churn candidates without meaningfully improving the protected set. A higher threshold like 12 or 16 would be a genuinely new experiment to try, since even lower thresholds haven't hurt performance much. On the other hand, I'm wondering about how items get re-inserted into probationary when they're demoted from protected—currently they go straight to the MRU end, which makes sense to avoid immediate eviction, but there might be a better approach based on their frequency history. only one item gets demoted per call, moving to the MRU end of probationary with a `was_protected` flag that gives it a quick re-promotion bypass on next access. I'm also considering an adaptive PROMO_THRESHOLD that adjusts based on protected segment utilization—raising it when near capacity to be more selective, lowering it when below 80% full—though with a 90/10 split at capacity, the protected segment stays nearly full anyway, so this might not provide much benefit. Now looking at tie-breaking strategies within the k=4 victim scan using LRU position.

The scan already handles ties correctly by starting from the LRU end and only updating the victim on strictly less-than frequency, so equal-frequency items naturally prefer the oldest one for eviction. I'm also exploring whether dividing probationary into distinct "warm" and "cold" zones could improve eviction decisions beyond the current global minimum-frequency approach.

Actually, the system already implements this implicitly: below-threshold items that get accessed move to the MRU end of probationary via `prob_push_back`, creating a natural warm zone of frequently-accessed items, while cold items drift toward the LRU end where the k=4 scan picks victims. So the current design already achieves a warm/cold split without explicit complexity.

Now I'm considering whether to scan from the MRU end instead of the LRU end—the MRU end has recently refreshed items that might be new cold entries unlikely to be re-accessed, whereas the LRU end contains items with no recent access history.

Actually, the current approach of scanning from the LRU end makes more sense: those are the oldest, coldest items, and picking the lowest-frequency among them targets the best victim. I could optimize this with an early-termination strategy—if I find a zero or very low frequency candidate during the scan, I can stop immediately rather than always checking k items.

Then there's the idea of making frequency-aware position updates within the probationary tier. Instead of always moving accessed items to the MRU end, I could place items with higher frequencies (but still below the promotion threshold) closer to MRU, and lower-frequency items further back. This creates a soft LRFU hybrid that gives priority to items approaching promotion. The tradeoff is that inserting at specific positions in the linked list becomes expensive without additional data structures like sorted buckets.

I also considered expanding the probationary zone from 10% to 15% or 20%, thinking that better frequency estimates might benefit from a larger probationary region. But earlier testing already settled on 90/10 as optimal for this workload, so that direction is closed.

Another idea: decay the sketch frequency for items that get re-admitted after eviction, since they carry historical frequency from their previous residence that might unfairly block newer items. But this would require tracking whether an item was previously evicted and adding generation counters, which complicates the structure significantly without a ghost set.

I'm also exploring whether the admission gate should compare incoming frequency against the median frequency of the scan candidates rather than just the victim's frequency—this would make admission harder to achieve and potentially improve selectivity.

Looking at what's genuinely worth pursuing, I should focus on re-probing the PROMO_THRESHOLD and victim scan window under the sharper baseline, plus investigating whether frequency-ordered eviction within the protected segment still has value. That last one was only tested once and might yield different results with better sketch estimates.

For the first hypothesis, I'm considering lowering PROMO_THRESHOLD from 8 to 5 using the improved sketch baseline. The threshold of 4 was tried before and regressed, but the baseline has shifted since then with deeper sketch estimates, so it's worth revisiting with fresh data. to 8 not yet tested) under maximum sketch quality, and H02 raising it to 12 to filter out mid-range frequencies that might be causing churn. But I realize these are both variations on the same parameter, so I should diversify—one hypothesis should test PROMO_THRESHOLD as the cross-cycle analysis suggests, another should re-probe victim scan width under the sharper sketch, and a third should explore frequency-ordered eviction within the protected set.

For H01, I'm testing PROMO_THRESHOLD at 5, a middle value between the regressed threshold of 4 and the current 8, to see if medium-frequency items benefit from protected status without the instability that lower thresholds caused. H02 shifts focus to widening the victim scan from k=4 to k=6, leveraging the improved frequency estimates to reconsider how many candidates we examine during eviction. was tested without the sharper sketch in cycle 0006/v001, so the cross-cycle notes suggest re-evaluating it under the improved sketch estimates. But adding SKETCH_DEPTH=6 in cycle 0008 didn't change hit rates, which makes me wonder if the estimates are actually different enough to matter. Let me think through whether the protected-segment approach would behave differently now.

The protected LRU tail becomes frequency-homogeneous because PROMO_THRESHOLD pre-filters items entering protected, so a k=4 scan rarely finds a colder candidate than the default LRU head. But with the sharper sketch (8× width + conservative update), the PROMO_THRESHOLD gate is more precisely applied, which could change how items at the threshold boundary (freq 8-10) are classified versus items well above it.

The per-policy notes mark this axis as "definitively closed," so maybe I'm looking in the wrong direction. Let me consider an alternative: what if the eviction path uses asymmetric tie-breaking when the minimum-frequency probationary candidate matches the protected head's frequency? Currently, eviction only considers probationary candidates and falls back to the protected head if probationary is empty, but it never compares the two segments directly when frequencies are equal.

If I scan the probationary LRU for the minimum-frequency victim and then compare it against the protected head, evicting whichever has lower frequency, that could improve hit rate when protected items have cooled down. But this seems counterintuitive—with PROMO_THRESHOLD=8, protected items should have frequency ≥ 8 while probationary items are typically lower. The real issue is that the sketch doesn't decay, so even cooled protected items retain their old counts.

Let me reconsider what's actually untested in the current baseline setup.

I could try a two-level frequency gate where the promotion threshold adapts to the median frequency of items currently in probationary, making it responsive to the actual workload rather than a fixed value. Or I could test intermediate threshold values like 5 or 6 to see if there's a sweet spot between the tested 4 and 8. There's also the possibility of weighting eviction candidates by both frequency and recency to avoid removing recently-added items, though that adds complexity. Another angle is to handle recently-demoted high-frequency items specially—if something gets demoted from protected but then accessed again in probationary, I could fast-track its re-promotion rather than making it climb back through the normal threshold. Another approach would be to modify the eviction scan so it avoids evicting items marked as `was_protected`, since those recently-demoted items might get re-accessed and re-promoted soon—instead targeting genuinely cold newcomers. I could also track a per-node access count since insertion to refine eviction decisions beyond just the sketch frequency, though that adds storage overhead.

Now I'm narrowing down to three concrete hypotheses that are distinct from each other and grounded in the open questions I've identified. The first one lowers the promotion threshold from 8 to 5, citing the cross-cycle winner, with the reasoning that a lower threshold lets medium-frequency items enter protected sooner and increase hit rate.

I'm reconsidering whether threshold=5 is actually a good intermediate value. The regression at threshold=4 in cycle 0007 might have been an overshoot rather than evidence the direction is fundamentally wrong, especially since the sketch is now more accurate with the 8× width and conservative update. But I need to trace back the exact lineage—cycle 0007's parent was cycle 0006/v002 with conservative update and 4× width, then v001 bumped to 8× width, and v002 tried lowering the threshold to 4. I'm getting tangled up in which version had which parent, so let me verify the actual dependency chain before finalizing this hypothesis. So threshold=4 under 8× width with conservative update is actually untested—it was only validated at 4× width. Threshold=6 is also unexplored under these conditions, making it a promising middle ground to test since the lower threshold regressed before. I'm planning to run threshold=6 as the next experiment under the full baseline setup.

For the victim scan width, k=6 sits between the tested k=4 (which was effective) and k=8 (which showed no improvement). The earlier conclusion that k=4 was saturated came from the noisier 4× width baseline, so under the sharper 8× width sketch, a slightly wider scan might actually yield better decisions without the overhead of k=8.

**H03: Victim admission gate with stricter comparison**

Actually, that would just make admission harder and likely hurt hit rate by rejecting too many newcomers. Let me think of something actually different.

**H03: Frequency-ordered eviction in protected segment under the full-accuracy baseline**

This was tested in cycle 0006/v001 before the sketch improvements and came back flat. But the cross-cycle analysis suggests it might be worth revisiting now that frequency estimates are sharper. The idea was to apply the same k=4 LRU-end scan logic to the protected segment — when it overflows, scan the k=4 LRU-end items and evict whichever has the lowest frequency.

With the current 8× width sketch and conservative updates, frequency estimates are much more accurate now. The PROMO_THRESHOLD=8 filter means protected items all started with frequency ≥ 8, but under sharper estimates I can now better distinguish between items that genuinely earned frequency 8-15 versus those with frequency 50+. Since frequencies don't decay in the sketch, items that cooled off still retain their old counts, so this differentiation might actually matter for eviction decisions now.

The real tension is that LRU-based demotion removes the oldest item in protected, which would have the highest sketch count from accumulated accesses over time. But a frequency-based scan would want to keep that high-count item and demote a newer, lower-frequency one instead—directly contradicting what pure LRU would do.

On a Zipfian trace, this could actually work in our favor: the very hot items accumulate enormous sketch counts, while less-hot items stay low. If a recently promoted item (freq=8) sits alongside an older, hotter item (freq=50), frequency-based demotion would correctly identify the newer one as the better candidate to remove, whereas LRU would evict the hot item just because it's oldest. The question is whether this behavior kicks in at the right time—when protected overflows after promoting a probationary item.

I should test this approach: instead of always demoting the LRU head of protected, scan a few items and pick the one with the lowest sketch frequency.

I'm setting this up as H03, and I also want to lay out two other hypotheses to test. H01 is lowering the promotion threshold from 8 to 6 under the full-accuracy baseline—threshold 4 regressed before, but 6 under the sharper estimates might hit the sweet spot. H02 is widening the victim scan to k=6, splitting the difference between the k=4 and k=8 values I've already tried.

For the parent references, I need at least one hypothesis to cite a cross-cycle winner. Since both cycle/0008/v001 and cycle/0007/v001 are winners, I'll use cycle/0008/v001 as the parent for H01 to satisfy that requirement, then vary the parents for H02 and H03 to keep the experiments diverse. I should use cycle/0008/v001 as the parent for all three hypotheses since it's the current best and most relevant baseline, even though it means less diversity in parent selection. For H03 specifically, using cycle/0008/v001 makes more sense than reverting to baseline, since that would lose the improvements already made in the recent cycles.

Now I'm reconsidering the k=6 parameter for victim scan. The earlier finding that k=8 showed no improvement over k=4 was under the noisier sketch, but with the sharper frequency estimates in cycle/0008/v001, the ranking of candidates might shift—items that appeared inflated before could now reveal their true lower frequencies, making k=6 a meaningful test point between the saturated k=4 and the previously ineffective k=8.

I'm also thinking about whether there's a completely novel direction beyond re-exploring PROMO_THRESHOLD and scan window size. One angle is adding a frequency floor at the admission gate itself—requiring incoming items to exceed not just the victim's frequency but also some minimum threshold like PROMO_THRESHOLD/2, which would prevent very cold newcomers from displacing only slightly less cold victims.

But the cross-cycle analysis already confirmed that TinyLFU's strict inequality handles one-hit wonders well, and raising the admission floor to frequency ≥ 2 was redundant, so that path is closed.

I'm also considering recency-weighted frequency for victim selection instead of raw sketch counts, but count decay was already tested and found harmful. And time-based probationary eviction—tracking insertion time and evicting the oldest items without access—would require timestamps we don't have.

Batched promotion with hysteresis is another angle: promoting items from probationary to protected in batches based on frequency ranking rather than per-access could reduce churn from borderline items bouncing between segments. It's complex and risky, but potentially worth exploring.

Alright, I'm settling on three solid hypotheses to test. The first one targets reducing PROMO_THRESHOLD from 8 to 6 to admit more medium-frequency items into protected and keep warm items from stalling in probationary.

The gap here is that threshold values other than 8 haven't been tested under the full 8× width baseline. With more accurate frequency estimates at this width, items truly in the 6-7 range should now be correctly identified as medium-hot. The second hypothesis widens the victim scan from k=4 to k=6

, which should help find lower-frequency eviction candidates more effectively under the sharper sketch estimates. The third hypothesis applies frequency-ordered demotion at the protected segment level, scanning k=4 candidates and demoting the one with minimum frequency rather than always the LRU head. So the frequency-ordered protected demotion was actually tested in cycle 0006/v001 without conservative update or the 8× width boost—those came later. Under the current baseline with both optimizations in place, this approach hasn't been evaluated yet, which is exactly what the open question is probing. Now I'm ready to draft the hypothesis files, though I should first verify that H01 holds up under the conditions we're testing.

Looking at cycle 0007/v002, setting PROMO_THRESHOLD=4 under conservative update plus 4× width caused a regression, suggesting the threshold was miscalibrated. Moving to threshold=6 sits between 4 and 8, but whether it helps depends on whether there are genuinely useful items in that 6-7 frequency band under the sharper sketch. With conservative update and 8× width, collision inflation drops significantly, so items with true frequency 5-7 should estimate accurately at 5-7 rather than inflating to 6-9 as they might have under 4× width. This means some genuinely hot medium-frequency items could now fall below threshold=8 when they previously would've qualified due to sketch inflation.

The key insight is that reducing collision inflation means fewer items get the artificial count boost that pushed them past the threshold=8 gate, so threshold=6 might actually help by catching items with true frequency 6-7 that no longer benefit from that inflation. I'll go ahead and write out the three hypothesis files now, using `/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses` as the output directory and confirming that `cycle/0008/v001` is the correct parent overlay path.

### 02:59:54 chat

Now I have a thorough understanding of the system. Let me synthesize the key findings before writing the hypotheses:

- **Best baseline**: cycle/0008/v001 (hit-rate=0.7435) — 8× sketch width + conservative update + SKETCH_DEPTH=6 + k=4 victim scan + PROMO_THRESHOLD=8 + 90/10 SLRU + was_protected bypass
- **Closed axes**: sketch aging, ghost-set, SLRU split ratios, sketch depth, hash-seed diversity, protected-segment scanning (under pre-sharpened sketch), admission-floor raising, direct-to-protected bypass
- **Open questions**: PROMO_THRESHOLD recalibration under the 8× width (threshold=4 was only tested under 4× width in cycle 0007/v002); k=6 scan (never tried, k=8 was flat pre-improvements); frequency-ordered protected demotion under the sharper sketch (only tried in cycle 0006 before conservative update + 8× width)

### 03:00:04 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nReducing PROMO_THRESHOLD from 8 to 6 will admit medium-frequency items into the protected segment that the current gate incorrectly excludes under the now-accurate sketch, raising hit rate.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the constant declaration from:\n\n```\nconst PROMO_THRESHOLD: u32 = 8;\n```\n\nto:\n\n```\nconst PROMO_THRESHOLD: u32 = 6;\n```\n\nNo other code changes. The `was_protected` bypass in `get()` is preserved unchanged. The admission gate in `put()`, the k=4 victim scan, and the 90/10 SLRU split are all left intact.\n\n### Rationale\n\nPROMO_THRESHOLD=8 was calibrated in cycle 0003 and carried forward. Threshold=4 was re-probed in cycle 0007/v002, but its parent was cycle/0006/v002 — the conservative-update baseline with the original 4× sketch width. At that narrower width, collision inflation was meaningfully higher: items with true frequency 5–7 frequently had their sketch estimates inflated to 8–10 by collisions, so they already cleared the threshold=8 gate automatically. The threshold=4 test regressed because it promoted genuinely second-tier items that were correctly blocked by threshold=8 under noisy estimates.\n\nUnder the cycle/0008/v001 baseline (8× width + conservative update), collision-inflated estimates are largely eliminated. Items with true frequency 6–7 now estimate accurately at 6–7 and are blocked by threshold=8. Threshold=6 is an untested intermediate that has never been evaluated against the full-accuracy sketch. If a non-trivial cohort of medium-hot items stalls in probationary because the sharper estimates correctly place them at freq 6–7 (rather than the previously inflated 8–10), lowering to 6 would let them enter protected and reduce unnecessary probationary churn.\n\n## Preservation invariants\n\n- `was_protected` bypass must remain unconditional (any item previously demoted from protected bypasses the gate regardless of frequency).\n- The admission gate in `put()` (`incoming_freq > victim_freq`) is unchanged.\n- The k=4 victim scan in probationary is unchanged.\n"
}
```

### 03:00:18 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nWidening the probationary victim scan from k=4 to k=6 will find lower-frequency eviction candidates more reliably under the now-accurate sketch estimates, raising hit rate.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the constant declaration from:\n\n```\nconst EVICTION_SCAN_K: usize = 4;\n```\n\nto:\n\n```\nconst EVICTION_SCAN_K: usize = 6;\n```\n\nNo other code changes. The victim selection loop in `put()` already iterates `for _ in 1..EVICTION_SCAN_K`, so changing the constant is the complete implementation. The scan still starts from `prob_head` (LRU end of probationary), selects the minimum-frequency candidate among the first k items, and falls back to `prot_head` if probationary is empty.\n\n### Rationale\n\nk=8 was tested in cycle 0005/v002 against the cycle 0004/v003 baseline (4× sketch width, no conservative update, no PROMO_THRESHOLD) and found flat — the insight concluded that \"the coldest reachable candidate already appears within the first four LRU-tail positions.\" That closure was reached with substantially higher collision noise in the sketch.\n\nUnder the cycle/0008/v001 baseline (8× width + conservative update), frequency estimates are accurate enough that candidate ranks 5–6 in the LRU tail may correctly show lower true frequencies that were previously masked by collision inflation. A k=6 scan has not been tested at any prior point in the evolution history. If even one additional position in ten correctly identifies a colder victim, the scan overrides an otherwise suboptimal eviction decision. k=6 (rather than k=8) is tested as the minimal extension that opens the new window without jumping to a value already confirmed flat.\n\n## Preservation invariants\n\n- The scan still originates from `prob_head` and traverses toward `prob_tail` (LRU to MRU order).\n- Tie-breaking remains first-found (LRU-biased): `victim_idx` updates only on strict `f < victim_freq`.\n- The admission gate (`incoming_freq > victim_freq`) and all promotion/demotion logic are unchanged.\n"
}
```

### 03:00:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nScanning k=4 protected-LRU candidates at demotion time and demoting the minimum-frequency one (instead of always the LRU head) will preserve higher-value protected residents under the sharper sketch estimates, raising hit rate.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, add a new constant after `EVICTION_SCAN_K`:\n\n```rust\n/// Number of LRU-end protected candidates to scan when selecting the demotion\n/// victim during protected-segment overflow.\nconst PROT_DEMOTION_SCAN_K: usize = 4;\n```\n\nThen replace the body of `maybe_demote_from_protected()`:\n\n**Before:**\n\n```rust\nfn maybe_demote_from_protected(&mut self) {\n    while self.protected_len > self.protected_capacity {\n        if let Some(h) = self.prot_head {\n            self.prot_unlink(h);\n            self.protected_len -= 1;\n            self.nodes[h].was_protected = true;\n            self.prob_push_back(h);\n            self.probationary_len += 1;\n        } else {\n            break;\n        }\n    }\n}\n```\n\n**After:**\n\n```rust\nfn maybe_demote_from_protected(&mut self) {\n    while self.protected_len > self.protected_capacity {\n        // Scan up to PROT_DEMOTION_SCAN_K items from the LRU end of protected\n        // and demote the one with the lowest estimated access frequency.\n        let Some(head) = self.prot_head else { break };\n\n        let mut victim_idx = head;\n        let mut victim_freq = self.sketch.estimate(&self.nodes[head].key);\n\n        let mut cursor = self.nodes[head].next;\n        for _ in 1..PROT_DEMOTION_SCAN_K {\n            let Some(c) = cursor else { break };\n            let f = self.sketch.estimate(&self.nodes[c].key);\n            if f < victim_freq {\n                victim_freq = f;\n                victim_idx = c;\n            }\n            cursor = self.nodes[c].next;\n        }\n\n        self.prot_unlink(victim_idx);\n        self.protected_len -= 1;\n        self.nodes[victim_idx].was_protected = true;\n        self.prob_push_back(victim_idx);\n        self.probationary_len += 1;\n    }\n}\n```\n\nNo other code changes. The `was_protected` flag is set on the demoted item as before, preserving the bypass on re-access. The probationary and protected list helpers are unchanged.\n\n### Rationale\n\nFrequency-ordered demotion from protected was explored once in cycle 0006/v001, against the cycle 0005/v001 baseline — before conservative update and before 8× sketch width. The cycle 0006 insight found it flat and concluded that PROMO_THRESHOLD pre-filters entrants, leaving the protected LRU tail frequency-homogeneous (all items have freq ≥ 8 at promotion time, so no colder candidate stands out). That argument was valid under a noisy 4× sketch: collision inflation blurred the frequency differences among protected-LRU residents.\n\nUnder cycle/0008/v001's 8× width + conservative update, protected residents' frequencies are estimated accurately. The protected LRU tail contains the oldest-resident items — items promoted earliest that have not been recently accessed. Among those, genuine frequency differences now surface correctly: some promoted at freq=8–10 (borderline at the gate, subsequently cold) vs. others at freq=50+ (durably hot). A k=4 scan within the protected LRU candidates at demotion time can now identify and demote the genuinely colder resident, preserving higher-value items in protected. The cross-cycle open question explicitly calls this out as untested under the sharper sketch.\n\n## Preservation invariants\n\n- The demoted item still receives `was_protected = true` and re-enters probationary at the MRU end (`prob_push_back`), preserving the re-promotion bypass on next access.\n- The scan is bounded to the protected list only and does not touch probationary nodes.\n- The outer `while self.protected_len > self.protected_capacity` loop is preserved; each iteration demotes exactly one item.\n"
}
```

### 03:00:50 chat

Here's a summary of the three hypotheses written to `/cycles/0009/hypotheses/`:

---

**H01** — `PROMO_THRESHOLD: 8 → 6` (parent: `cycle/0008/v001`)

The key insight: threshold=4 was only ever tested under 4× sketch width (cycle 0007/v002), where collision inflation was still significant. Under the current 8× width + conservative update, items with *true* frequency 6–7 now estimate accurately at 6–7 instead of being inflated to 8–10. Those items are newly blocked by the threshold=8 gate that didn't exist before. Threshold=6 is a never-tested intermediate that could rescue medium-hot items now stalling in probationary.

---

**H02** — `EVICTION_SCAN_K: 4 → 6` (parent: `cycle/0008/v001`)

k=8 was flat in cycle 0005 but that was against a noisy 4× sketch where candidate ranks 5–8 had inflated estimates and appeared equivalent. Under the sharper 8× + conservative-update baseline, accurate frequency ranking could differentiate candidates 5–6 from 1–4. k=6 is an untested intermediate that probes whether the scan's effective range has grown now that estimates are more precise.

---

**H03** — k=4 frequency-ordered demotion from protected (parent: `cycle/0008/v001`)

`maybe_demote_from_protected()` is modified to scan the 4 LRU-end protected items and demote the minimum-frequency one rather than always the LRU head. This was tried in cycle 0006/v001 (before 8× width and conservative update) and found flat because PROMO_THRESHOLD made the protected LRU tail frequency-homogeneous. Under the sharper sketch, real frequency differences among protected-tail residents are now correctly estimated, potentially revealing items that were promoted at freq≈8 but have since cooled — which the cross-cycle insights flag as the highest-priority un-revisited structural question.


---

- finished: 2026-06-06T03:00:50+00:00
- duration: 363.69s
- status: ok
- input_tokens: 12
- output_tokens: 18568
- cache_read_tokens: 349619
- cache_write_tokens: 46471
- cost_usd: 0.559125
