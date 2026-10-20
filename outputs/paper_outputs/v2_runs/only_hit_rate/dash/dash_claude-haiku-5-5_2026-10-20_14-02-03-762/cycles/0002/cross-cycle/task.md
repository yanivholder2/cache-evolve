# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three changes to how items move between the probing, stash, and normal tiers: where a stash-promoted item lands in its target bucket, whether a probing hit re-inserts the item, and whether foreign evictions skip the retry in their true target. Only v002, the change to the landing slot for stash hits, won; the other two regressed on hit-rate and were not winners.

## Patterns

- Landing position matters more than movement. Moving a re-referenced stash item to the MRU end of its target bucket was the only change that helped, while the two changes that added movement or removed a retry both hurt hit-rate.
- Keeping residency in the probing tier appears valuable. Removing and re-inserting probing hits pushed proven-hot keys back through target competition and lost ground.
- Shortening the demotion path alone does not pay off. Skipping the target retry for foreign evictions lowered the one measured dimension, and the predicted per-op-time gain was never recorded.
- The winner's evidence is thin. Its only measured dimension is hit-rate, on a single workload, so the win is a hit-rate result, not a demonstrated overall improvement.
- Rationale is inconsistently documented around the winning change. The Front/Back default is described differently in different places, which makes the intent of v002 harder for later cycles to verify.

## Open questions

- Does the MRU landing for stash hits hold on workloads other than the LIRS trace used here?
- Do per-op-time and metadata-size move at all under any of these changes? No variant measured them.
- Would the removed target retry pay for itself if the shorter eviction path is paired with better stash capacity handling?

## Continuation

Next cycle should build on v002: vary the stash-hit landing slot further and measure it across more workloads, plus per-op-time and metadata-size. Drop the probing-hit remove-and-reinsert idea and the direct-to-stash eviction change unless a follow-up explicitly measures the costs they were meant to offset.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried three changes around stash promotion and foreign-eviction placement: gating stash promotion on a second hit, making the stash-hit landing slot depend on target-bucket occupancy, and restoring the target retry for foreign evictions only when the stash is full. Only v001 is listed as a Winner, and its measured hit-rate matched its parent, so the win is inherited rather than new.

## Patterns

- The one clear hit-rate gain came from admission filtering. Requiring a second hit before a stash item is promoted helped on the LIRS trace, which suggests one-off stash hits were displacing resident items.
- Refinements of the landing slot changed nothing measurable. The occupancy-conditional landing matched its parent, so the cohort gives no evidence that the full-bucket case it targets is common on this workload.
- Gating the direct-to-stash shortcut on free stash capacity recovered only a small part of the hit-rate lost by the unconditional shortcut. That supports the view that stash pressure is what made the removed retry valuable.
- Every variant was judged on hit-rate alone. Per-op-time and metadata-size were measured in none of them, so no cost of the promotion filter or its marker is known.
- Several insights rest on the hypothesis text rather than the overlay diff, so the implementations are not confirmed as described.

## Open questions

- Is v001 a Winner against the baseline, or only against its parent v002? Its own insight reports no change from that parent.
- Does the second-hit promotion filter hold on workloads other than the LIRS trace?
- Does the one-bit seen-in-stash marker stay within existing entry metadata, and what does it cost in per-op-time?
- Is the occupancy-conditional landing ever triggered on any workload?

## Continuation

Next cycle should build on the second-hit promotion filter: test it on more than one workload and measure per-op-time and metadata-size alongside hit-rate. Drop the occupancy-conditional landing refinement, since it showed no effect, and treat the capacity-gated direct-to-stash path as a secondary lever to revisit only if the promotion filter's cost is measured.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
