# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/hypotheses
- allow_edits: True
- started: 2026-06-06T02:32:42+00:00

## timeline

### 02:32:46 thinking

Let me read all the context files to understand the current state of the research and formulate 3 distinct hypotheses for cycle 0007.

### 02:32:46 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/*/"
}
```

### 02:32:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/cycle-winners.json"
}
```

### 02:32:47 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/cycle-insight.md"
}
```

### 02:32:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/leaderboard.md"
}
```

### 02:32:49 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cross-cycle-insights.md"
}
```

### 02:32:51 thinking

Now let me read the cycle 0006 variants to understand what was tried and what worked:

### 02:32:52 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v*/"
}
```

### 02:32:52 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles-ledger.jsonl"
}
```

### 02:32:55 thinking

Now let me read the actual overlay files for cycle 0006 variants to understand what code changes were made:

### 02:32:56 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/**/*"
}
```

### 02:32:56 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/baseline/**/*"
}
```

### 02:32:58 thinking

Let me read the key overlay files and insight files to understand what changes have been made:

### 02:32:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v002/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:33:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v002/insight.md"
}
```

### 02:33:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v001/insight.md"
}
```

### 02:33:00 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v003/insight.md"
}
```

### 02:36:58 thinking

Now I have a thorough understanding of the current state. Let me summarize:

**Current best variant: cycle/0006/v002** (hit-rate=0.7428)
- Uses conservative CMS update (key change from v002)
- SLRU 90/10 split
- k=4 victim scan in probationary
- PROMO_THRESHOLD=8 with was_protected bypass

**What's been tried and closed:**
- Sketch aging at any cadence → harmful (closed)
- Ghost-set re-admission → harmful (closed)
- Admission window → harmful (closed)
- Put-path promotion symmetry → no effect (closed)
- Scan window widening (k=4→k=8) → no effect (closed)
- Protected segment victim scanning → no effect (closed)
- Split-ratio beyond 90/10 → no effect (closed)
- Admission floor increase → redundant (closed)

**Open questions for cycle 7:**
Whether sketch width and depth could improve signal quality, how the conservative-update gain interacts with aging mechanisms, and whether that gain holds up on non-Zipfian workloads.

**Remaining tuning opportunities:**
Frequency-aware ordering within the protected segment instead of LRU, and the admission gate threshold which hasn't been explored yet.

**Testing three hypotheses:**

Hypothesis 1 builds on the best result from cycle 6 by doubling the CMS sketch width factor from 4× to 8× capacity to reduce hash collisions and sharpen frequency estimates.

Hypothesis 2 is still being formulated. I'm realizing the distinction between victim scanning in protected (which searches the LRU tail for minimum frequency) versus maintaining frequency-sorted ordering within protected itself. The cycle 6 insight noted that frequency was already homogeneous at the LRU tail, which is why scanning didn't help — but I'm questioning whether sorting by frequency instead of access time would actually change the outcome if that homogeneity persists regardless of ordering strategy. The v001 approach of selecting the minimum-frequency item for demotion didn't help because the LRU head was already the coldest candidate in the protected tail. So frequency ordering within protected seems exhausted too. For cycle 7, I should focus on sketch parameters instead—doubling the width from capacity × 4 to × 8 could reduce hash collisions and improve frequency estimates, or increasing depth from 4 to 6 or 8 for better accuracy.

The PROMO_THRESHOLD value of 8 is another unexplored lever; lowering it might keep the protected region more exclusive by reducing promotions. I'm also reconsidering the TinyLFU admission gate itself—whether the comparison operator or the threshold needs adjustment, though cycle 6 testing suggested the floor of 2 didn't help.

Looking at the clearest optimization paths: widening the CMS sketch by doubling the width factor would reduce hash collisions and overcounting, especially paired with the conservative update already in place. Alternatively, increasing sketch depth from 4 to 6 rows would tighten frequency estimates by adding more hash functions, though I'm weighing whether this trades off against the current setup's coverage.

But the more promising angle is tuning the promotion threshold itself—with conservative update producing tighter frequency counts, the current threshold of 8 might be too aggressive, preventing genuinely hot items from being promoted. Lowering it to 4 or 5 would directly leverage the sharper estimates we're getting from the conservative update strategy.

I'm also considering a different approach: capping the sketch's maximum counter value at something like 64 instead of letting it grow unbounded. This creates implicit aging where very hot items plateau naturally, allowing new bursts to displace them without needing explicit periodic halving. The key distinction is that saturation just caps growth rather than actively erasing counts—which aligns with the finding that accumulated frequency is signal, not noise.

But there's a tension here: u32::MAX already provides enormous range, and capping at 64 would lose the ability to distinguish between items accessed 64 times versus 1000 times. For Zipfian traces though, the frequency gap between hot and cold is so large that anything above 64 might be "hot enough" for admission decisions. It's a promising hypothesis but carries the same risk as aging—potentially harmful.

Another angle is adding hysteresis to the admission gate: instead of just requiring incoming > victim, demand incoming > victim + delta for some positive delta. This creates a threshold that prevents marginal items from constantly displacing each other.

Actually, that would just make admission harder and tank hit rate. Let me zero in on the concrete levers: sketch width hasn't been tuned, and with conservative update now giving sharper counts, the PROMO_THRESHOLD calibration might be off. The old noisy sketch probably inflated frequencies, so a key appearing at freq=8 back then might only be freq=5-6 now. Lowering the threshold to 4 or 5 could restore the original intended promotion behavior.

For cycle 7, the highest-value move is sketch configuration—width and depth sizing plus reset frequency are all untouched. I'm considering wider CMS (SKETCH_WIDTH_FACTOR 4→8) and deeper CMS (SKETCH_DEPTH 4→6) as separate experiments, but those are too similar. The third option should be something distinct, like lowering PROMO_THRESHOLD.

But there's another angle worth exploring: building from baseline instead of inheriting from the conservative-update lineage. The cross-cycle insights suggest that frequency-aware ordering within the protected segment—maintaining the list sorted by frequency so MFU items stay protected and LFU items get demoted first—might succeed where victim scanning in protected failed. This is structurally different from cycle 6's approach, which only scanned candidates at eviction time without reorganizing the list itself.

However, the insight flags that this would likely result in frequency-homogeneous items at the LRU tail anyway, so the gain would probably be zero. I'll skip this direction.

What I'm actually interested in now is reducing PROMO_THRESHOLD from 8 to 4. With conservative update tightening the sketch counts, items that genuinely get accessed 4-5 times might not reach the old threshold of 8 anymore. Lowering it could let more medium-hot items enter the protected segment, which could help with the second tier of the Zipfian distribution and improve overall hit rate.

But wait — cycle 0002 already tried threshold=2 and it failed. Then cycles 0003/0005 found threshold=8 to be effective. The cycle-5 insight notes that items clear threshold=8 quickly on this trace, meaning the gating isn't actually synergistic with the other mechanisms. Lowering to 4 might be too permissive and let in too many cold-ish items instead.

The real question is whether conservative update tightens the frequency estimates for borderline items (those around 5-10 accesses), potentially keeping them out of protected when they should be promoted. If that's the case, I should lower the threshold to compensate for the more conservative estimates.

Let me now settle on the three hypotheses to test.

**H01: Widen the sketch** — doubling the sketch width from 4× to 8× capacity reduces hash collisions, which should sharpen frequency estimates even further when combined with conservative update.

**H02: Lower the promotion threshold** — since conservative update produces tighter frequency estimates, items that previously hit threshold=8 might now only register at 5-6. Dropping the threshold to 4 recalibrates the gate to match the new, more conservative scale and lets genuine medium-hot items into protected.

**H03:**

For the next hypothesis, I'm weighing whether to test deeper sketches (6 hash functions instead of 4) with standard update as a baseline experiment, isolating the depth effect from conservative update. The advantage is it explores a different research angle and provides a control. But the risk is that without conservative update, baseline depth might not improve much — and the cross-cycle insight suggests conservative update is now the new baseline, so testing depth alone could be less valuable than testing it alongside conservative update.

I'm reconsidering H03 entirely: instead of just adding depth, what if I implement a decaying saturation cap on the CMS counters? Rather than letting them grow unbounded, I'd cap them at a lower threshold like 64 or 128. This creates implicit aging where hot keys saturate and fresh bursts can displace them more quickly, which is conceptually different from sketch sizing and addresses a distinct mechanism for handling frequency estimation.

But I'm second-guessing this approach since the cross-cycle data shows aging and counter halving have consistently regressed—saturation might hit the same wall despite being mechanically different from halving.

Let me explore another angle: what if I make insertion position in the probationary segment frequency-aware? When a new item enters during put(), instead of always going to the MRU end, I could position it based on its pre-insertion frequency—items that were previously hot would get inserted closer to MRU to survive longer before eviction. This leverages the frequency signal at insertion time rather than treating all new items equally.

Actually, looking at the current code, new items already go to the MRU end during put(), and the k=4 scan reads from the LRU end, so a hot newcomer would naturally survive longer anyway.

The third option is raising PROMO_THRESHOLD from 8 to 16. The idea is that with conservative update giving sharper frequency counts, items reaching 16 are genuinely hotter, making the protected segment more exclusive and reducing churn. But cycle-5 showed that gating tends to collapse toward unconditional promotion in practice—items clear threshold=8 quickly. Raising it to 16 might force some items through probationary multiple times before qualifying, which hasn't been tested yet. It's in the same direction as cycle 0002's threshold=2, which was too low.

With conservative update, counts become more conservative, so reaching threshold=8 requires slightly more real accesses. This could meaningfully change behavior at the margin, unlike the current setup where PROMO_THRESHOLD feels compatible but not synergistic with unconditional promotion.

So my three hypotheses are: wider CMS sketch (SKETCH_WIDTH_FACTOR 4→8) to reduce collisions and sharpen estimates, deeper CMS sketch (SKETCH_DEPTH 4→6) for lower estimation error and better frequency discrimination, and raising PROMO_THRESHOLD to 16 to make the protected segment more selective.

Actually, the first two are too similar in targeting sketch quality. Let me pivot: H01 stays as wider sketch, H02 becomes lowering PROMO_THRESHOLD to 4 to recalibrate the promotion gate for conservative-update estimates, and H03 should test something fundamentally different — maybe frequency-weighted admission where incoming items must exceed victim frequency by some threshold.

But that approach would likely just make admission harder and hurt hit rate. Let me reconsider what I'm actually trying to explore here, especially given the constraint that H03 can branch from baseline or prior non-winners.

Looking at what's been definitively ruled out versus what remains untested, I see that sketch dimensions, PROMO_THRESHOLD values like 4 and 16, and the semantics of the TinyLFU admission gate are still open. The frequency-aware ordering in the protected set probably won't help based on the scan results, but PROMO_THRESHOLD tuning from the baseline might be worth revisiting with different values.

I'm considering a different angle entirely—soft admission that lets low-frequency items enter probationary on a hit. But first, let me examine the actual get() implementation to understand what's happening during lookups and promotions.

The code checks if an item is in the protected region and refreshes its position if so, otherwise it evaluates whether the frequency meets the promotion threshold before deciding whether to promote to protected or just refresh within probationary. For the put() path, there's a frequency-sampled victim selection that compares incoming frequency against candidate victims. An unexplored direction could be tweaking the hash seed strategy with Fibonacci hashing to improve the sketch's behavior.

Another angle is implementing a doorkeeper pattern using a Bloom filter before the CMS — this is actually part of the standard TinyLFU design. Items would only get counted in the CMS on their second encounter, which saves CMS capacity for truly frequent items and reduces collision noise. The current design already handles one-hit-wonders at the admission gate, but a doorkeeper would prevent them from ever touching the frequency estimator in the first place. The doorkeeper approach would underestimate frequencies on first visits, potentially hurting promotion decisions since items wouldn't reach the threshold as quickly. Plus it adds complexity with key storage requirements. Let me try a simpler angle instead—using recency as a tie-breaker in the victim selection and admission gate, so when frequencies are equal, I prefer evicting the older item to admit the newcomer.

This is different from adding a minimum admission frequency threshold. On stable Zipfian workloads where new items typically have lower frequencies than established residents, ties would be rare anyway, so this change might not help much. But I'm also considering a more sophisticated approach: instead of just picking the minimum-frequency victim in the k=4 scan, I could weight candidates by both age and frequency together.

The idea is to score each candidate as frequency multiplied by its position in the scan, where position 0 is the LRU head (oldest) and increases toward the MRU end. This creates a hybrid metric that penalizes items that are both old and frequent while favoring newer items even if they have decent frequency. So an old item with frequency 3 gets a low score, while a newer item with frequency 4 gets a higher score, making the newer one less likely to be evicted.

Now I'm settling on three concrete hypotheses to test: widening the CMS sketch to reduce collisions, lowering the promotion threshold to make frequency updates more conservative, and adjusting the eviction policy itself. — baseline parent avoids confounding with the conservative-update effect. But I'm realizing that if baseline itself doesn't include conservative update, this hypothesis might miss an important interaction. Let me step back and reconsider what H03 should actually test. The cross-cycle insights point to sketch width, depth, and reset interval as the highest-priority untuned axes now that conservative update is the new baseline, so maybe H03 should focus on depth rather than recency-weighted victim selection, which would be a clearer orthogonal strategy.

I'm settling on three distinct hypotheses: H01 increases sketch width from 4 to 8 inheriting conservative update from the prior winner, H02 tunes the promotion threshold down from 8 to 4 also with conservative update, and H03 tests sketch depth from 4 to 6 against baseline to isolate the depth effect without conservative update confounding it. These cover different tuning axes, though I'm noting that threshold tuning has been explored before in earlier cycles with various values, so I need to check what the current baseline is actually using.

With conservative update in place, the frequency counts are more accurate and lower than before, which means items that appeared to have freq=8 under standard update might actually be freq=4-5 now. This changes how the threshold gate functions—raising it to 12 or 16 might create more meaningful protection than lowering it, since the inflated counts that made threshold=8 permissive are no longer happening. The cross-cycle insights confirm the threshold isn't exhausted, just that the current design yields marginal benefit. With conservative update shrinking frequency estimates, I'm settling on two concrete experiments: widening the sketch from 4 to 8 to reduce hash collisions alongside the conservative update, and recalibrating PROMO_THRESHOLD down to 4 since the lower frequency estimates mean the original threshold of 8 is now effectively too strict.

I'm also testing deeper sketches (4 to 6 hash functions) as a baseline to isolate whether depth alone provides sufficient discrimination without conservative update. These three hypotheses target distinct mechanisms—sketch width, gate calibration, and sketch depth—and all reference valid parent cycles. Now I need to set up the output directory and write out the experiment files.

For each hypothesis, I'm making targeted code changes: H01 doubles the sketch width factor, H02 lowers the promotion threshold to 4, and H03 increases the sketch depth to 6 hash functions.

Now I need to figure out what the baseline code actually looks like since I don't have the source files — I only see the evaluation results. Looking at the H03 variant, it uses conservative update logic, but I need to understand whether the baseline uses standard or conservative update semantics to properly set up H03 as a variant. I should set H03 to use cycle/0006/v002 as the parent instead, adding depth=6 as the isolated change while keeping the conservative update. This way I have two sketch-dimension experiments (width and depth) plus one policy experiment, which aligns with the cycle insights recommending both width and depth tuning. So my three hypotheses become: H01 testing width increase, H02 testing promotion threshold decrease, and H03 testing depth increase—all building on the same conservative baseline.

Now I'm wondering if having two sketch structure changes (width and depth) makes them too similar or if they're distinct enough since they test different dimensions. They could produce different results depending on which dimension actually matters, so I think they're sufficiently varied, though I could explore making H03 something more structurally different if needed.

The cross-cycle insights point toward aging policy or cache-size ratios as potential frontiers, but those are either already closed or outside what I can test. Instead, I'm considering a different approach: using key-derived hash seeds rather than row-index seeds in the CMS hash function to see if that changes how collisions are distributed.

Currently each row uses its index as the seed, which means keys with similar hash patterns might collide in correlated ways across rows. By replacing those sequential seeds with fixed constants derived from mathematical constants like the golden ratio and prime numbers, I can make each row's hash function more independent and reduce those correlated collisions — this is a standard technique in CMS design.

So I'm settling on three distinct improvement strategies: expanding the sketch width to directly reduce collision rates, recalibrating the promotion threshold to work better with the tighter frequency estimates from conservative updates, and improving hash independence through better seed constants. All three target different aspects of the algorithm and build on the cross-cycle winner.

I need to be careful with H03 since Rust's DefaultHasher has implementation-defined randomization, but the current approach of writing a seed value before hashing the key is straightforward to modify. I'm going to write out these three hypotheses now in the output directory, starting with H01 which focuses on the sketch width increase and its effect on collision rates and frequency estimation.

For H01, I'm doubling the width factor constant from 4 to 8, which scales up the sketch dimensions and reduces collisions without touching any of the conservative update logic or promotion threshold. For H02, I'm lowering the promotion threshold from 8 down to 4 to better calibrate the admission gate for the more conservative frequency estimates that come from the CMS update strategy. Updating the hash function... Defining hash seed constants... Updating the function calls to pass the row index directly instead of casting it, and ensuring the HASH_SEEDS array stays in bounds by using modulo if needed. Now I'm ready to write out the implementation.

### 02:36:58 chat

Now let me read the baseline source files to understand the starting code:

### 02:36:58 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/inputs/subjects/tiny_lfu/**/*"
}
```

### 02:36:59 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0006/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs"
}
```

### 02:37:14 thinking

Now I have a very clear picture of both the baseline (v001 from cycle 0006, which tested the protected-segment scan) and the winning v002 code (with conservative update). 

Let me also note what v001 has vs v002:
- v001: Has a k=4 scan in `maybe_demote_from_protected()` — but this produced zero gain
- v002: Uses conservative update in `increment()`, doesn't have the k=4 scan in `maybe_demote_from_protected()`
- The cycle/0006/v002 overlay (the cross-cycle winner) is the one I should build on
- I'm testing two main directions: doubling the sketch width factor to reduce hash collisions, and lowering the promotion threshold from 8 to 4 to account for how conservative updates compress frequency estimates

. I'm also considering replacing the sequential row seeds with independent hash constants to decorrelate the sketch rows and improve minimum estimation accuracy. All three approaches target different aspects of sketch quality and reference the proven baseline from the cross-cycle winner. Now I'm setting up the output and writing these variations.

### 02:37:14 chat

Now I have everything I need. Let me create the output directory and write the three hypothesis files:

### 02:37:15 tool: Bash

```json
{
  "command": "mkdir -p /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/hypotheses",
  "description": "Create cycle 0007 hypotheses directory"
}
```

### 02:37:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/hypotheses/H01.md",
  "content": "# H01\n\nDoubling the CountMinSketch width factor from 4× to 8× capacity halves the expected per-row hash collision rate, sharpening frequency estimates further beyond what conservative update alone provides to raise hit rate.\n\n## Parent\n\ncycle/0006/v002\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the module-level constant:\n\n```rust\nconst SKETCH_WIDTH_FACTOR: usize = 4;\n```\n\nto:\n\n```rust\nconst SKETCH_WIDTH_FACTOR: usize = 8;\n```\n\nNo other changes are needed. The computed sketch width in `try_new()` —\n`(capacity * SKETCH_WIDTH_FACTOR).max(MIN_SKETCH_WIDTH).next_power_of_two()` —\ndoubles automatically for any capacity ≥ 4, giving a sketch width of 8× capacity\nrounded up to the next power of two (e.g., 16 384 columns for capacity=2048).\n\nAll other logic inherited from cycle/0006/v002 is preserved unchanged: the\nconservative CMS update in `increment()`, `PROMO_THRESHOLD=8` with the\n`was_protected` bypass in `get()`, the k=4 minimum-frequency victim scan in\n`put()`, the simple LRU-order demotion in `maybe_demote_from_protected()`, and the\n90/10 protected/probationary split in `try_new()`.\n\n## Rationale\n\nConservative update (cycle/0006/v002) reduced the effect of hash collisions by\nonly incrementing rows at the current minimum, but it cannot eliminate collisions\n— two keys sharing a counter cell still inflate each other's estimates.  A wider\nsketch reduces the probability that any two keys hash to the same position in a\ngiven row, cutting the base collision rate in half.  The two mechanisms are\ncomplementary: width reduces how often collisions happen; conservative update\nlimits how much they inflate estimates when they do happen.  Combining a wider\nsketch with conservative update should produce tighter frequency estimates than\neither change alone, enabling the k=4 victim scan and PROMO_THRESHOLD gate to\nmake fewer wrong decisions.\n"
}
```

### 02:37:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/hypotheses/H02.md",
  "content": "# H02\n\nLowering PROMO_THRESHOLD from 8 to 4 recalibrates the promotion gate for the tighter frequency estimates produced by conservative CMS update, admitting genuinely medium-hot items that now stall below the original threshold to raise hit rate.\n\n## Parent\n\ncycle/0006/v002\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, change the module-level constant:\n\n```rust\nconst PROMO_THRESHOLD: u32 = 8;\n```\n\nto:\n\n```rust\nconst PROMO_THRESHOLD: u32 = 4;\n```\n\nNo other changes are needed. The promotion check in `get()` —\n`if freq >= PROMO_THRESHOLD || self.nodes[idx].was_protected` — uses this\nconstant directly; the `was_protected` bypass remains intact and unchanged.\nAll other logic (conservative update, width factor=4, k=4 victim scan,\n90/10 split) is preserved.\n\n## Rationale\n\n`PROMO_THRESHOLD=8` was originally calibrated against standard CMS update, which\ninflates estimated frequencies via hash collisions.  An item with true access\ncount 5–6 could reach an estimated frequency of 8 simply because it shared\ncounter cells with other keys, clearing the threshold with artificially inflated\ncounts.  Conservative update eliminates this inflation: it only increments rows\nalready at the minimum, so the estimate stays close to the true minimum.  An item\nthat would have shown estimated freq=8 under standard update might now show\nestimated freq=4–5 under conservative update — and never qualify for promotion\nunder the unchanged threshold.  These are precisely the medium-hot items in the\nsecond tier of the Zipfian distribution (visited several times but not among the\ntop percentile) that belong in protected.  Lowering PROMO_THRESHOLD to 4 restores\nthe intended admission behaviour under the new, more accurate frequency scale\nwithout making the gate trivially open (items must still have been accessed at\nleast 4 times before promotion).\n"
}
```

### 02:37:57 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-38-25-817/cycles/0007/hypotheses/H03.md",
  "content": "# H03\n\nReplacing sequential row-index seeds (0, 1, 2, 3) with diverse fixed-constant seeds in the CountMinSketch hash function reduces correlated collisions across rows, tightening the minimum frequency estimate to raise hit rate.\n\n## Parent\n\ncycle/0006/v002\n\n## Changes\n\nIn `src/tiny_lfu/mod.rs`, add a new module-level constant array of four well-separated\nu64 seeds, one per sketch row:\n\n```rust\nconst HASH_SEEDS: [u64; SKETCH_DEPTH] = [\n    0x9e3779b97f4a7c15,  // Knuth multiplicative / golden-ratio constant\n    0x6c62272e07bb0142,  // FNV-1a prime-derived constant\n    0x517cc1b727220a95,  // large prime used in splitmix32\n    0xbf58476d1ce4e5b9,  // splitmix64 first mixing step constant\n];\n```\n\nModify the `position()` helper to accept a `row: usize` argument (replacing\n`seed: u64`) and look up the corresponding constant:\n\n```rust\nfn position<K: Hash>(&self, key: &K, row: usize) -> usize {\n    let mut hasher = DefaultHasher::new();\n    hasher.write_u64(HASH_SEEDS[row]);\n    key.hash(&mut hasher);\n    (hasher.finish() as usize) & (self.width - 1)\n}\n```\n\nUpdate both callers to pass `row` instead of `row as u64`:\n\n- In `increment()`: change `let pos = self.position(key, row as u64);` to\n  `let pos = self.position(key, row);`.\n- In `estimate()`: change `.map(|(row, _)| self.counts[row][self.position(key, row as u64)])`\n  to `.map(|(row, _)| self.counts[row][self.position(key, row)])`.\n\n`SKETCH_DEPTH` remains 4, so `HASH_SEEDS[row]` is always in-bounds.  All other\nlogic inherited from cycle/0006/v002 is preserved unchanged: conservative update\nin `increment()`, `PROMO_THRESHOLD=8` with `was_protected` bypass, k=4 victim\nscan, 90/10 split.\n\n## Rationale\n\nThe current `position()` uses the row index itself (0, 1, 2, 3) as the seed\nwritten before the key.  Because the seed values are small sequential integers,\nkeys with similar internal hash structure can produce correlated positions across\nmultiple rows — both hashing to the same column in row 0 and to the same column\nin row 1.  When two keys collide in every row their estimates contaminate each\nother equally across all rows, and the minimum-based estimate provides no benefit\nover any single row.  Using four diverse, well-separated seed constants from\nestablished hash-mixing sequences breaks this correlation: keys that collide under\none seed are very unlikely to collide under a structurally different seed,\nrestoring the independence assumption the CMS minimum estimator depends on.  This\nis orthogonal to both conservative update (which limits per-collision inflation)\nand sketch width (which reduces collision probability); it targets the case where\ncollisions, if they do occur, cluster across all rows simultaneously.\n"
}
```

### 02:38:07 chat

All three hypothesis files are written. Here is a summary of what was produced:

---

**Cycle 0007 — 3 Hypotheses**

All three build on **cycle/0006/v002** (hit-rate=0.7428, the cross-cycle winner), which introduced conservative CMS update. The cycle-6 insight explicitly recommended sketch configuration (width/depth) as the highest-value next probe, and noted the promotion-gate calibration may need revisiting in light of the new tighter estimates.

**H01 — Wider sketch (SKETCH_WIDTH_FACTOR 4→8)**  
Doubles the sketch width from 4×capacity to 8×capacity columns. Conservative update limits how much collisions inflate estimates; a wider sketch reduces how often collisions occur in the first place. The two effects are orthogonal and expected to compound.

**H02 — PROMO_THRESHOLD recalibration (8→4)**  
`PROMO_THRESHOLD=8` was calibrated when standard CMS update inflated counts. Conservative update keeps estimates close to the true minimum, so a medium-hot item (true freq ≈ 4–6) that previously cleared threshold=8 through inflation may now stall in probationary indefinitely. Lowering to 4 restores the intended promotion behavior on the new, accurate frequency scale.

**H03 — Independent row hash seeds**  
The current `position()` uses sequential integers (0, 1, 2, 3) as row seeds, which can produce correlated column positions across rows for structurally similar keys. Replacing these with four widely-spaced constants from established hash-mixing sequences (golden ratio, FNV, splitmix) breaks inter-row collision correlation — the condition the CMS minimum estimator depends on for tightness. Only the four seed constants and `position()`'s parameter type change; all other inherited logic is preserved.


---

- finished: 2026-06-06T02:38:07+00:00
- duration: 325.02s
- status: ok
- input_tokens: 11
- output_tokens: 16791
- cache_read_tokens: 342934
- cache_write_tokens: 44333
- cost_usd: 0.522466
