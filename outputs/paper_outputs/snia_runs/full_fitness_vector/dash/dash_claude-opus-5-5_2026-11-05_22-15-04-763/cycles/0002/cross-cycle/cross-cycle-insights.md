## Recurring wins

- Hit rate improves with a two-hit gate on promotion from stash to the normal tier. This held in both cycles, and every cycle-0002 variant built on the gate still beat the baseline on hit rate.
- Changes that keep proven or recently demoted items around longer reliably raise hit rate. Examples: sending promotions into free probing space instead of evicting a resident, and demoting normal-tier victims to the stash MRU end.
- Large-size hm_0 is the most sensitive configuration in both cycles. It gains the most when re-referenced items are kept and loses the most when they are dropped. Use it as the first place to check any retention idea.
- Reusing the existing unused per-item field (`lfu_counter`) for gate state cost no extra metadata. Prefer this over adding new fields.

## Recurring failures

- Tail latency: no variant has yet improved p99 in either cycle. Each hit-rate gain came with a small tail regression, for a different reason each time: the promote/demote cascade, longer lookups, and stash occupancy pressure.
- `put` slowed down in both cycles and has never been attributed to stash fullness versus benchmark noise.
- Skipping the MRU move on the first stash hit gave a slightly better p50, the same p99, and some lost hit rate. Treat this direction as closed.
- In cycle 0001, the simulator build failed on Maven/Gradle 429 rate limits and two variants were never measured. Infrastructure failures can silently shrink a cohort.

## Open questions

- Do MRU demotion and probing overflow stack on top of the gate? Might probing overflow's fewer demotions offset the stash pressure that MRU demotion adds?
- Can the third-bucket lookup cost of probing overflow be removed, for example through search order or migrating items back to target, while keeping its hit-rate gain?
- Is the `put` p99 regression actually caused by stash occupancy? It needs a run that separates the effect from noise.
- Can the latency of the second-hit cascade be reduced directly, for example with balanced target/probing promotion? Balanced promotion was proposed in cycle 0001 but has never been measured.

## Per-policy notes

- Dash (stash + normal tier):
  - Current base: two-hit gate plus MRU demotion into stash. It has the strongest hit rate so far, at the cost of a small tail regression.
  - The gate's latency cost comes from the second-hit cascade and lookup length, not from the first-hit path.
  - Promotion placement (where promotions land) and demotion placement (where demotions land) are independent levers. Each helps hit rate on its own, and they have not been tested together.
