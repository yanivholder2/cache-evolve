# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses
- allow_edits: True
- started: 2026-06-05T18:49:34+00:00

## timeline

### 18:49:37 thinking

Let me understand the task:
1. I'm a hypotheses-writer for an evolving cache research project
2. I need to generate 3 distinct hypotheses for cycle 0005
3. Each hypothesis must cite exactly one parent variant (baseline or cycle/<NNNN>/v<NNN>)
4. At least one must cite the cross-cycle winner: cycle/0004/v001 with hit-rate=0.684
5. I need to read context files to understand what has been tried and learned
6. Output goes to /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses

Let me start by reading the context files in the specified order:

1. First, read the cycle 0004 results to understand what was tried
2. Read the cycle-level summary
3. Read the cross-cycle history if present
4. Then synthesize 3 distinct hypotheses grounded in what I've learned

Let me start reading the files.

### 18:49:37 chat

I'll help you generate 3 distinct hypotheses for cycle 0005. Let me start by reading the context files to understand what has been tried and learned so far.

### 18:49:38 tool: Bash

```json
{
  "command": "find /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants -type d -name \"v*\" | sort",
  "description": "List all variants in cycle 0004"
}
```

### 18:49:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cycle-insight.md"
}
```

### 18:49:38 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cycle-winners.json"
}
```

### 18:49:39 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/leaderboard.md"
}
```

### 18:49:40 thinking

Good, I'm starting to get a picture. Let me now read the actual variant details from cycle 0004 to understand what changes were made, and then check the cross-cycle insights.

### 18:49:41 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v*/insight.md"
}
```

### 18:49:41 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cross-cycle-insights.md"
}
```

### 18:49:44 thinking

Good, now I have a clearer picture of what has been tried. Let me get the details of the cycle 0004 variants to see what exactly was changed in each. I'll need to read the overlays and understand what changes v001 made.

### 18:49:45 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001/insight.md"
}
```

### 18:49:45 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001/attempts/*/overlay"
}
```

### 18:49:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v002/insight.md"
}
```

### 18:49:46 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v003/insight.md"
}
```

### 18:49:47 thinking

Good, now I have a much clearer picture. Let me read the actual overlay files to understand what code changes were made. The Glob didn't find the attempts directory, so let me try a different approach to locate those files.

### 18:49:48 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v*"
}
```

### 18:49:48 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles-ledger.jsonl"
}
```

### 18:50:09 thinking

Excellent! Now I have a comprehensive picture of what has been tried and what succeeded/failed:

## Summary of findings across cycles:

**What worked:**
1. Stash-hit acceleration (promoting stash-hit items) - consistent winner
2. MRU back insertion for demoted items - improved hit rate in cycle 0003
3. Deterministic tier-skip for stash hits

**What didn't work:**
1. Combining MRU insertion for both promotion AND demotion (cycle 0004 v001) - asymmetry revealed
2. Intra-tier MRU repositioning within stash (cycle 0004 v002) - orthogonal to tier advancement
3. Combining multiple strategies (cycle 0004 all failed) - strategies interfere at higher tiers

**Current best:** cycle/0004/v001 with hit-rate=0.684

Wait, let me re-read this. The cycle-winners.json shows:
- v001: 0.684 (winner)
- v002: 0.6293
- v003: 0.6785

But the cycle-insight.md says cycle 0003's v003 had 0.6839. So the best so far is 0.684 from cycle 0004/v001, which actually has the same hit rate as the best variant from earlier cycles. The insight mentions that ALL variants in cycle 0004 "hurt hit rate compared to cycle 0003's winner" but v001 at 0.684 is slightly BETTER than v003's 0.6785 in cycle 0004.

Looking at the leaderboard:
- baseline: 0.663
- cycle 0001: 0.678, 0.663, 0.671
- cycle 0002: 0.678, 0.668, 0.656


- cycle 0003: 0.678, 0.671, 0.684
- cycle 0004: 0.684, 0.629, 0.678

So 0.684 appears in both cycle 0003 v003 and cycle 0004 v001. The v001 strategy applied back insertion to both demotion and promotion, which seems counterintuitive given it matched the previous best. Let me check if there's something I'm missing about how this actually performed relative to its peers.

The insight clarifies that universalizing MRU positioning across both paths doesn't create the expected synergy—the two approaches plateau at the same level rather than amplifying each other. So v001's 0.684 is essentially equivalent to v003's 0.684 from the prior cycle, making cycle/0004/v001 the current cross-cycle leader.

For cycle 0005, I need three distinct hypotheses with at least one referencing cycle/0004/v001. The analysis points to a potential local optimum—the cycle insight questions whether 0.684 represents a genuine plateau or if there's still room to escape it through a different approach.

The core tension is MRU's asymmetry: it helps demotion but degrades fresh promotion. Rather than continuing to tweak insertion positions, I should test fundamentally different mechanisms—tier capacity constraints, stash eviction policies, bucket structure changes. The guidance is clear: stop combining previously successful strategies and instead investigate what's driving the asymmetry or pivot to entirely different levers like eviction policies and tier sizing.

For cycle 0005, I need three distinct hypotheses: one building on the cycle 0004 winner to probe the MRU asymmetry more carefully, and two exploring different mechanisms entirely. The demotion-only back insertion angle seems redundant since cycle 0003 v003 already tested that. What I should focus on is understanding why adding promotion back insertion alongside demotion back insertion creates interference—that's the real asymmetry to investigate. use that as parent for at least one hypothesis.

The real question is what to build on from v001 since it matches v003's performance at 0.684. The promotion back insertion change that hurt performance might actually be instructive—perhaps there's a different approach to handling fresh promotions that's less disruptive than immediate MRU placement.

Testing a delayed or middle-ground insertion position for promoted items could reveal whether the issue is the asymmetry between immediate MRU positioning versus forcing items to re-earn status at their new tier. Beyond that, I should explore completely different mechanisms like tier capacity constraints, stash eviction policies, and bucket structure variations, which represent untested levers for improvement.

For stash eviction specifically, being more selective about which items leave—preserving hot items while only removing cold ones—might extend their time to prove value. Similarly, adjusting the relative sizes of stash, target, and probing tiers could address whether items are getting stuck due to capacity constraints or not getting enough opportunity to demonstrate their worth.

I need to check the overlay files to see what changes were actually implemented in each version.

### 18:50:09 chat

Now let me read the detailed implementation changes from the cycle 0004 variants to understand exactly what was modified:

### 18:50:09 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001/attempts/01"
}
```

### 18:50:09 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0003/variants"
}
```

### 18:50:11 thinking

The glob patterns aren't finding directories. Let me try a different approach - let me look at the actual directory structure to find the overlay files.

### 18:50:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v001/attempts/01/variant-implementer.log.md"
}
```

### 18:50:11 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v002/attempts/01/variant-implementer.log.md"
}
```

### 18:50:12 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/variants/v003/attempts/01/variant-implementer.log.md"
}
```

### 18:50:51 thinking

Excellent! Now I have a complete picture of what has been tried in cycle 0004:

**Cycle 0004 variants:**
1. **v001 (winner, 0.684)**: Combining back insertion for demoted items AND target-tier promotions. The insight reveals this created asymmetry problems - MRU insertion works for demoted items but destabilizes newly promoted items.

2. **v002 (0.6293)**: Repositioning accessed items within stash on each access. This hurt hit rate because intra-tier ordering within stash doesn't translate to hit-rate improvement - items need to be promoted to higher tiers, not just reordered within stash.

3. **v003 (0.6785)**: Combining stash-hit tier-skip (skip target, go to probing) with back-insertion demotion. This hurt hit rate because the tier-skip strategy assumes filtering by intermediate tiers, but back-insertion demotion undermines this discipline by giving demoted items a fast-track.

**Cross-cycle winners:** cycle/0004/v001 with hit-rate=0.684

**Key learnings from cycles 0001-0004:**
- Stash-hit acceleration works (deterministic or adaptive)
- MRU positioning on demotion works
- Fitness signals (stash-hit, recency) gate promotion
- Deterministic rules outperform conditional branching
- Combining successful strategies creates interference - hard boundaries reached
- MRU asymmetry: works for demotion, hurts for fresh promotion
- Local optimum concern: is 0.684 really the peak?

**Open questions for cycle 0005:**
1. Is 0.684 a local optimum or is there headroom?
2. Why the MRU asymmetry?
3. Are there completely different levers (tier capacity, eviction policy, bucket structure) that could help?

Now I need to generate 3 distinct hypotheses for cycle 0005:


- At least one MUST cite cycle/0004/v001 as parent (the cross-cycle winner)
- Each hypothesis should explore a distinct strategy
- They should be grounded in what has been learned

Let me think about 3 distinct hypotheses:

### H01: Investigate the MRU asymmetry - test demotion-only back insertion (parent: cycle/0004/v001)
Actually, cycle/0003/v003 already tested demotion-only back insertion and got 0.684 same as v001. So this would be redundant. Better approach: test what's special about v001 if it's the same as v003's performance. I should isolate whether the target-tier promotion logic in v001 is actually helping or if it's just noise. Let me test v001 but remove the target-tier promotion and keep only the back-insertion demotion to see if performance stays at 0.684 or drops.

Actually, a better direction: **Test promotion insertion delay** - parent: cycle/0004/v001. Instead of immediately placing promoted items at MRU, introduce a delay or eligibility window before they can be promoted. This tests whether the asymmetry works because promoted items need to "earn" their position rather than jumping straight to the front. Or apply the promotion asymmetry more carefully by testing whether promoted items should enter at a middle position instead of MRU.

### H02: Explore fundamentally different lever - stash eviction policy (parent: baseline)
Rather than tweaking insertion order, change the eviction strategy itself. Test whether evicting the coldest items from stash first while preserving hot items longer produces better results. This shifts focus away from promotion/demotion mechanics to a different control point.

### H03: Explore tier capacity/sizing (parent: baseline)
Adjust the relative sizes of tiers to test whether stash capacity is the bottleneck. If stash is undersized, items may not get sufficient time to demonstrate access patterns before being evicted.

Looking back at the cycle insight, v001 scored 0.684 versus v003's 0.6785, making v001 the winner. But the v001 insight reveals that applying back insertion to both promotion and demotion paths created interference—promoted items competing poorly with the tier's eviction cadence. This suggests universalizing MRU positioning across both paths didn't synergize as expected, which seems contradictory to v001 being the best performer.

The key realization is that v001 and v003 essentially saturated at the same level—the two strategies didn't amplify each other, they just plateaued. For cycle 0005, I should stop combining previously successful approaches since they've hit their interaction limits. Instead, I need to investigate the fundamental asymmetry: why does the MRU mechanism help demotion recovery but harm fresh promotion? Testing position-based promotion eligibility delays or demotion-specific insertion policies that don't apply to forward promotion could help break through this ceiling. The cycle 0003 baseline might be a local optimum, so if asymmetry variations don't yield gains, I should pivot to entirely different levers like eviction policies, tier sizing, or stash capacity thresholds rather than continuing to refine insertion position logic. Intra-tier optimization isn't the bottleneck here.

Now I'm generating three distinct hypotheses to test. The first explores whether the issue is immediate access to further promotion by delaying promotion eligibility for newly promoted items, forcing them to "prove themselves" at the higher tier before advancing again. The second abandons insertion position entirely and instead tests selective cold-item eviction from stash to see if occupancy management is the real constraint. The third adds a time or access-count qualification period before items can be promoted from stash to target, testing whether the promotion problem stems from timing rather than positioning.

I need to clarify the baseline hierarchy though—the permanent reference is 0.663, but cycle/0004/v001 achieved 0.684 as the cross-cycle winner, so that becomes the required parent for at least one hypothesis. Let me restructure: H01 should use v001 as parent and test promotion eligibility delay by examining whether the problem is eviction cadence, promotion candidacy, or something else entirely.

For newly promoted items, I'd add a minimum access count or recency requirement before they're eligible for further tier advancement—this directly isolates the "promotion candidacy eligibility" variable from the cycle insight. H02 takes a different angle using baseline as parent: instead of uniform LRU eviction from stash, implement selective eviction that targets truly cold items (high frequency but long untouched) while preserving hot items longer, which should reduce hit-rate loss from premature stash eviction. I'm realizing these are getting unwieldy though—I should focus on what's actually implementable: insertion position during promotion/demotion, time-based windows, and eviction policy tweaks.

Let me narrow to three concrete hypotheses. H01 addresses the MRU asymmetry by inserting freshly promoted items at LRU position, forcing them to "prove themselves" at the new tier before becoming promotion candidates again—this makes promotion more disciplined while keeping demotion recovery aggressive. H02 tests occupancy-gated promotion from stash, where only items meeting certain occupancy thresholds get promoted, providing an alternative filtering mechanism to the standard approach.

H03 reduces the probing tier capacity by 25% to force more aggressive demotion and give target-tier items better opportunity to establish themselves before promotion, testing whether the current sizing ratio is optimal for maintaining higher-quality residents.

However, the tier structure appears fairly rigid based on cycle insights, making H03 potentially difficult to implement. Let me refocus on more tractable approaches.

H01 directly addresses the MRU asymmetry by marking newly promoted items as ineligible for further promotion for one access cycle, allowing them to build residency discipline at their new tier while retaining demotion recovery. This isolates whether the issue stems from eviction cadence or premature promotion eligibility.

H02 gates stash-to-target promotion on stash occupancy thresholds, only allowing promotion when occupancy drops below a target level. This tests whether the filtering discipline problem is fundamentally about occupancy-based gatekeeping rather than tier-skip velocity.

H03 applies back insertion selectively—only for stash-to-target promotions while keeping target-tier promotions at front insertion. This explores whether the asymmetry lies in how different promotion pathways should be positioned, though it risks overlapping with prior attempts.

The real pattern emerging is that singular, focused mechanisms outperform composite ones. Asymmetry between demotion and promotion appears critical—the same positioning strategy that helps one can undermine the other. So the next cycle should isolate one clear asymmetry-based mechanism building on the v001 winner, test one entirely new lever, and refine one narrower scope variant. H01 takes the asymmetry approach by keeping demotion's back insertion while reverting promotion to front, isolating which direction actually benefits from MRU positioning.

H02 explores a different constraint: making newly promoted items ineligible for immediate re-promotion until they've been accessed again, which forces them to earn their place at probing rather than just gaining insertion advantage. H03 shifts to an unexplored variable—using stash occupancy as a gate for promotion eligibility, where high stash occupancy triggers direct promotion to probing, bypassing the intermediate step.

### 18:50:51 chat

Now I have a comprehensive understanding of the cycles. Let me read a couple more files to ensure I have all context needed:

### 18:50:51 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/inputs/subjects/dash/baseline"
}
```

### 18:51:02 thinking

That's okay, I don't need the baseline directory itself. I've gathered enough context. Let me now synthesize the 3 hypotheses for cycle 0005 based on everything I've learned.

Key insights to guide hypothesis generation:
1. At least one MUST cite cycle/0004/v001 (the cross-cycle winner at 0.684)
2. The best so far is 0.684 - is it a local optimum or can we do better?
3. Cycle 0004 revealed hard boundaries when combining strategies
4. The key asymmetry is: MRU works for demotion, hurts for promotion
5. The continuation guidance says to either:
   a) Investigate the asymmetry through variations (position-based promotion eligibility delays, demotion-specific policies)
   b) Or pivot to fundamentally different levers (eviction policies, tier sizing, stash capacity thresholds)

Let me create 3 distinct hypotheses:

### H01: Asymmetry refinement - demote-only back insertion (parent: cycle/0004/v001)
Build on v001 (the winner). v001 tried to apply back insertion to both promotion AND demotion, which created interference. Test the hypothesis that we should apply back insertion to demotion ONLY, without the promotion side. This isolates the asymmetry by keeping what works (demotion recovery) and removing what doesn't (promotion MRU).

Specific change: In demote_from_normal_bucket, keep using InsertPosition::Back for all demoted items (from v003). But in promotion_get, when promoting to probing, use InsertPosition::Front instead of applying back insertion to promoted items. This tests whether the interference comes from over-extending MRU to promotion paths.

### H02: Promotion eligibility delay (parent: cycle/0004/v001)
The cycle insight asks: "What is the precise mechanism that makes MRU insertion beneficial for demoted items but harmful for freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, or something else?"


Newly promoted items should enter a "cold" state where they can't immediately compete for further promotion—they need to be accessed again at their current tier first. This enforces more disciplined promotion without sacrificing demotion benefits. Specific change: Mark promoted items with a flag indicating they haven't yet "earned" their tier position, blocking further promotion until accessed again. This tests whether rapid successive promotions are the actual problem.

### H03: Stash occupancy-gated promotion threshold (parent: baseline)
Test a fundamentally different lever: occupancy-based gating. Only promote items from stash when stash occupancy is below a threshold (e.g., < 70%). When stash is heavily occupied, accessed items stay in stash (are repositioned to MRU but not promoted). This tests whether the filtering discipline identified in v003's failure is about proportional occupancy rather than insertion position.

Specific change: In promotion_get, when a stash hit is found, check stash occupancy. If below threshold, promote to target (baseline behavior). If above threshold, reposition to back (MRU) and return from stash without promoting. This tests whether the issue is premature promotion under pressure vs. allowing items to mature in stash first.

These three are:
1. **H01**: Asymmetry refinement (builds on v001, tests demotion-only vs universal back insertion)
2. **H02**: Asymmetry investigation (builds on v001, tests promotion eligibility mechanism)
3. **H03**: Different lever entirely (builds on baseline, tests occupancy-gated filtering)

Let me write these into files now.

### 18:51:02 chat

Perfect! Now I have all the context I need. Based on my analysis of cycles 0001-0004, I can now synthesize 3 distinct hypotheses for cycle 0005. Let me write them to the output directory:

### 18:51:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H01.md",
  "content": "# H01\n\nRefine MRU insertion asymmetry by applying back insertion to demotion paths only, preserving v003's proven demotion recovery while testing whether promotion-path back insertion (the new part of v001) is the source of interference.\n\n## Parent\n\ncycle/0004/v001\n\n## Changes\n\nModify the `promotion_get()` function in `src/dash/segments/dash_segment.rs` to test whether back insertion belongs in demotion paths only.\n\n**For demotion**: Preserve the back-insertion logic from v003 for all demotion paths (target→stash, probing→target→stash cascades). Demoted items continue to use `InsertPosition::Back` to maintain promotion eligibility after surviving tier discipline.\n\n**For promotion**: Change promotion insertions to use `InsertPosition::Front` (LRU) instead of the universal back insertion attempted in v001. Specifically:\n1. In `promotion_get()`, when promoting stash hits to target (line ~165), use `InsertPosition::Front` \n2. When promoting hottest target items to probing (line ~177), use `InsertPosition::Front`\n\nThis isolates the working part of the asymmetry (demotion recovery) from the part that may be interfering (promotion-path MRU insertion). The hypothesis tests whether v001's saturation at 0.684 is due to back insertion on promotion paths over-accelerating items through tiers.\n\n## Rationale\n\nCycle 0004 v001 revealed that applying back insertion universally (both promotion and demotion) saturated at 0.684 instead of improving. The cycle insight questions whether the MRU mechanism's asymmetric behavior is rooted in insertion position or in candidacy timing. By reverting promotion paths to Front insertion while preserving demotion-path back insertion, this variant isolates which direction of movement benefits from MRU treatment. If hit rate remains at 0.684, it confirms that demotion recovery (not promotion acceleration) is the mechanism; if it drops, it confirms that both paths contribute equally and the saturation is due to a different cause.\n"
}
```

### 18:51:13 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H02.md",
  "content": "# H02\n\nInvestigate promotion candidacy asymmetry by delaying promotion eligibility for newly promoted items, requiring them to prove fitness at their new tier before becoming candidates for further promotion.\n\n## Parent\n\ncycle/0004/v001\n\n## Changes\n\nExtend the MRU asymmetry investigation with an eligibility-delay mechanism in `src/dash/segments/dash_segment.rs`.\n\n**Track promotion recency**: Add a promotion-access counter to items (could use existing metadata or a parallel tracking structure). When an item is promoted to a new tier (target or probing), mark it with promotion-age = 0.\n\n**Gate further promotion**: In `promotion_get()`, modify the hottest-item promotion logic (target→probing) to skip promotion of items with promotion-age < 1 (i.e., items promoted to target in the immediately preceding access cycle). These items must be accessed again at their new tier before competing for further promotion.\n\n**Preserve demotion**: Demoted items land at Back (from v001's parent v003) and immediately have promotion-age = 1, preserving their re-entry opportunity. This differentiates demoted items (which have proven history) from freshly promoted items (which are unproven at the new tier).\n\n**Specific implementation**: \n1. Keep v001's demotion-path back insertion intact\n2. When promoting from stash→target or target→probing, record the promotion timestamp or access epoch\n3. In promotion-candidate selection for target→probing, exclude items promoted fewer than 1 access-cycle ago\n4. This creates a \"prove-yourself\" window at each tier for newly promoted items, without affecting demoted items' rapid re-entry\n\n## Rationale\n\nThe cycle 0004 insight asks whether the promotion-path MRU problem is \"eviction cadence, promotion candidacy eligibility, or something else.\" This variant directly tests the eligibility hypothesis: is the asymmetry caused by items being too quickly eligible for further promotion after arriving at a tier? By making newly promoted items ineligible for one access cycle while keeping demoted items eligible, this separates the two populations and tests whether enforced residency windows resolve the asymmetry without blocking demotion recovery.\n"
}
```

### 18:51:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H03.md",
  "content": "# H03\n\nTest a fundamentally different lever: occupancy-based promotion gating. Gate stash-to-target promotion on stash occupancy, allowing items to mature in the stash tier when occupancy is high, addressing the filtering discipline problem identified in cycle 0004 v003.\n\n## Parent\n\nbaseline\n\n## Changes\n\nModify the stash-hit path in `promotion_get()` in `src/dash/segments/dash_segment.rs` to implement occupancy-aware promotion gating.\n\n**Occupancy threshold check**: When a stash hit is detected (line ~155), before promoting to target, check the stash bucket's current occupancy ratio relative to its capacity.\n\n**Conditional promotion logic**:\n1. If stash occupancy is below 70%: promote the item to target with `InsertPosition::Front` (baseline behavior), allowing natural tier progression\n2. If stash occupancy is at or above 70%: reposition the item to Back (MRU) within the stash bucket and return without promoting, keeping it in the stash tier\n\n**Rationale for the threshold**: At high occupancy, the stash is filtering effectively—items that survive 70%+ occupancy have proven value. Promoting them immediately risks pushing marginal items into target/probing. By keeping high-occupancy survivors in stash, we extend their proving period and reduce promotion of unproven items.\n\n**Preserve all other paths**: This change affects only the stash-hit promotion decision. All other tier transitions (target→probing, demotion cascades) follow baseline behavior.\n\n## Rationale\n\nCycle 0004 v003 failed when combining tier-skip with back-insertion demotion, revealing that \"tier-skipping mechanism assumes a filtering discipline that back-insertion undermines by giving demoted items a shortcut.\" This suggests the filtering discipline—allowing items to survive tier transitions and prove themselves—is the real lever, not insertion position alone. By gating promotion on occupancy, we enforce filtering: items accessed during high-occupancy periods stay in stash longer, while low-occupancy periods allow free tier progression. This tests whether the local optimum at 0.684 is constrained by insufficient filtering, not by insertion position conflicts.\n"
}
```

### 18:51:25 thinking

Perfect! I've successfully created all 3 hypotheses for cycle 0005. Let me verify they meet the quality bar:

✅ **Distinct strategies**:
- H01: Asymmetry refinement (demotion-only back insertion, tests isolation)
- H02: Asymmetry investigation (promotion eligibility delay, tests candidacy timing)
- H03: Different lever entirely (occupancy-gated promotion, tests filtering discipline)

✅ **Testable**:
- H01: Can measure if reverting promotion paths to Front changes behavior
- H02: Can measure if adding eligibility delay after promotion helps
- H03: Can measure if occupancy-gated filtering improves hit rate

✅ **Grounded**:
- H01: Cites cycle/0004/v001 (the cross-cycle winner, as required)
- H02: Cites cycle/0004/v001 (tests the identified asymmetry deeper)
- H03: Cites baseline (tests a completely different lever)

✅ **Specific in Changes**:
- H01: Specifies which functions to modify and which insertion positions to change
- H02: Specifies promotion-age tracking, eligibility-gate logic, and preservation of demotion
- H03: Specifies occupancy threshold (70%), conditional behavior, and which paths are affected

✅ **Requirement met**:
- At least one (actually two) cite the cross-cycle winner cycle/0004/v001 ✓

The hypotheses are well-grounded in the cross-cycle insights and cycle-specific learnings, directly addressing the open questions raised by the framework:
- H01/H02 investigate the MRU asymmetry mechanism
- H03 tests the "fundamentally different levers" suggested in the continuation
- All three avoid the failed patterns (intra-tier optimization, occupancy-only without fitness signals)


- Done.

### 18:51:26 chat

Perfect! I've successfully generated 3 distinct hypotheses for cycle 0005. Let me verify they meet the quality bar:

### 18:51:26 tool: Glob

```json
{
  "pattern": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H*.md"
}
```

### 18:51:27 thinking

Perfect! All three hypothesis files have been created successfully. Let me do a final validation by reading one of them to make sure the format is correct.

### 18:51:27 chat

Excellent! Let me verify the format is correct by spot-checking one:

### 18:51:27 tool: Read

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0005/hypotheses/H01.md",
  "limit": 30
}
```

### 18:51:32 thinking

Perfect! The format is correct. The file has:
1. ✅ A claim (line 2-3): "Refine MRU insertion asymmetry by applying back insertion to demotion paths only..." - this is one clear sentence starting with an action verb
2. ✅ ## Parent section (lines 5-7): Contains exactly one identifier: cycle/0004/v001
3. ✅ ## Changes section (lines 9-19): Provides specific implementation details about which functions to modify, what lines to change, and how the behavior differs
4. ✅ Optional ## Rationale section (lines 21-23): Explains why this test matters

All requirements are met. The hypotheses are:
- **Distinct**: H01 is asymmetry refinement, H02 is asymmetry investigation, H03 is different lever
- **Testable**: Each can be evaluated against the hit-rate metric
- **Grounded**: Two cite v001 (the required cross-cycle winner), one cites baseline
- **Specific**: Clear function names, line numbers (approximate), and implementation details

The task is complete!

### 18:51:36 chat

Perfect! All three hypotheses are properly formatted and written to the output directory. Let me provide a summary of what was generated:

## Summary: Cycle 0005 Hypotheses Generated

I've successfully synthesized **3 distinct hypotheses** grounded in the cross-cycle insights:

### **H01: Asymmetry Refinement** — Parent: `cycle/0004/v001` (the cross-cycle winner)
Tests whether back insertion is needed in **demotion paths only**, not promotion paths. v001 applied back insertion universally and saturated at 0.684. This variant isolates which direction benefits from MRU treatment by reverting promotion paths to Front insertion while preserving demotion-path back insertion from v003.

### **H02: Promotion Eligibility Delay** — Parent: `cycle/0004/v001`
Directly investigates the identified MRU asymmetry mechanism. The cycle insight questions whether the problem is "promotion candidacy eligibility" rather than insertion position. This variant requires newly promoted items to be accessed once at their new tier before competing for further promotion, while preserving demoted items' immediate re-entry opportunity.

### **H03: Occupancy-Gated Promotion** — Parent: `baseline`
Tests a fundamentally different lever (as suggested in cycle 0004 continuation). Gates stash-to-target promotion on stash occupancy: only promote when occupancy < 70%, otherwise keep accessed items in stash at MRU. This enforces the "filtering discipline" identified as missing in cycle 0004 v003's tier-skip failure.

---

**Quality validation:**
- ✅ **Distinct strategies**: Three independent approaches (asymmetry refinement, asymmetry investigation, different lever)
- ✅ **Testable**: Each produces measurable hit-rate outcomes
- ✅ **Grounded**: H01 & H02 cite the required cross-cycle winner; H03 explores the suggested "fundamentally different levers"
- ✅ **Specific**: Function names, line numbers, insertion positions, occupancy thresholds—all concrete
- ✅ **Requirement met**: At least one (actually two) hypothesis cites `cycle/0004/v001`

The hypotheses are ready for variant implementation and evaluation.


---

- finished: 2026-06-05T18:51:37+00:00
- duration: 123.00s
- status: ok
- input_tokens: 68
- output_tokens: 11491
- cache_read_tokens: 440390
- cache_write_tokens: 68321
- cost_usd: 0.1883
