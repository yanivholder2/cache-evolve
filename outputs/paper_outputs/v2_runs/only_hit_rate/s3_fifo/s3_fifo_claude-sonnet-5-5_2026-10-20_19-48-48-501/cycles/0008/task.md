# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_19-48-48-501/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three structurally different main-queue and ghost-path tweaks were tried: a one-time reinsertion grace for ghost-admitted entries (v001), protected-end insertion of high-freq small-queue victims into main (v002), and a per-key freq flag in the ghost, with re-admission at freq=2 (v003). None improved hit-rate. v002 is the Winner only because it tied its parent, while v001 and v003 were marginally worse.

## Patterns

- Nothing moved hit-rate. This is the second cycle running in which the best variant merely tied the parent. The S3-FIFO structure looks close to a local optimum on this trace.
- Both variants that gave ghost-admitted keys extra credit (a grace pass, a boosted freq) lost slightly. Ghost-hit information is already used well by plain admission, and extra protection keeps entries that stop being reused.
- The main insertion position (v002) looks like a weak lever. Either few victims reach the threshold, or main's reinsertion logic washes out the position difference.
- Variant authors still could not read the overlay source, so we cannot tell whether v002's change was a no-op. The tie is ambiguous.
- Metadata-size and per-op-time were again missing, and v003 added ghost state with no signal. Evidence is still one trace at one cache size.

## Open questions

- Did v002 alter behaviour at all, or is it equivalent to its parent?
- Is any remaining lever on this trace worth the effort, or are we measuring noise?
- Do these null results hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored so cost can be compared?

## Continuation

Treat v002 as a neutral base; prefer the simplest parent if it is cleaner. Drop ghost-hit bonuses, per-key ghost metadata, and main insertion-position tweaks. The next cycle should validate on a second trace and cache size, and restore the metadata and per-op-time measurements. Only try new levers (such as main-eviction scanning or reinsertion rules) if the diff is checked to change behaviour.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three variants were tried. Two changed main-queue eviction: a bounded lookahead that picks the lowest-freq entry (v001), and a cap on reinsertions per eviction (v002). The third (v003) combined known admission settings: main evictees go to the ghost, a very small small queue, and a shorter ghost. Only v003 won, with a hit-rate gain. Both eviction variants lost a little.

## Patterns

- Every change to main-eviction mechanics lost hit-rate. Replacing reinsertion with lookahead lost, and capping reinsertion depth also lost. The unbounded second-chance reinsertion scan is doing real protective work.
- The one Winner is a consolidation of earlier admission gains (small queue shrink, shorter ghost), plus ghost insertion of main evictees. It adds no per-key state. This again points to admission sizing, not eviction, as the lever that moves this trace.
- Ageing freq in place for skipped entries (v001) fits the older finding that ageing and decay changes lose here.
- Cost was still unmeasured. Variant authors still could not read overlay source. Evidence is one trace at one cache size, so small regressions may be noise.

## Open questions

- Which part of v003 carries the gain: ghost insertion of main evictees, or the re-bundled sizing?
- Does v003's extra ghost work on `put` cost meaningful per-op time?
- Does the gain hold on a second trace or cache size?

## Continuation

Adopt v003 as the base. Drop main-eviction rewrites (lookahead, reinsertion caps, freq ageing) and keep the parent's reinsertion semantics. Next, isolate whether ghost insertion of main evictees helps, and validate on a second trace and cache size. Restore metadata-size and per-op-time measurements so cost can be compared.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three probes of the sizing-plus-ghost configuration were tried: a stricter small-to-main promotion test (freq>=2, v001), ghosting only main evictees that showed reuse (v002), and an ablation of main-evictee ghosting from the small-queue/short-ghost sizing (v003). v001 is the Winner only because it tied its parent. v002 lost a very small amount, and v003 gained over baseline.

## Patterns

- Sizing alone (small queue ~5%, shorter ghost) delivers the hit-rate gain over baseline. Main-evictee ghosting is not needed for it, so the cycle 0006 gain was mostly sizing.
- Filtering main evictees out of the ghost lost slightly. Evictees that look unreused still carry some re-admission value, and no ghost pollution showed up.
- A stricter promotion gate (freq>=2) again made no measurable difference. This is the second cycle with a tie on promotion strictness, so the gate is either not binding on this trace or the change is a no-op.
- Ghost content and ghost admission selection, like capacity ratios, look saturated. Every cycle's best variant has been a tie or a marginal gain.
- Variant authors still could not read overlay source. Cost dimensions (metadata-size, per-op-time) are still unmeasured. Evidence is one trace at one cache size.

## Open questions

- Does the freq counter ever reach 2 while an item sits in small? If not, the v001 tie is a no-op rather than evidence.
- How much does main-evictee ghosting add on top of sizing? v003 shows it is not required, but not whether it helps or hurts.
- Do the sizing gains hold on a second trace or cache size?

## Continuation

Treat the simplest sizing-only configuration (v003) as the base; it is as good as the more complex 0006 winner, so prefer it unless ghosting main evictees is shown to add something. Drop promotion-threshold and ghost-content filtering variants. Next, validate the sizing on a second trace and cache size, restore metadata-size and per-op-time measurements, and only try new levers if the diff is verified to change behaviour.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
