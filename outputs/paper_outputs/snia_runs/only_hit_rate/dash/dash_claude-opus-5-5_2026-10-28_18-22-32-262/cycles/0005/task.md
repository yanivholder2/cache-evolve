# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle-0001 v001 (MRU-end stash demotion), and each added a different stash lever. v001 added a fixed two-hit exit gate, v002 added a TinyLFU-style frequency comparison against the target's LRU victim, and v003 inserted fresh puts at the stash LRU end with a BIP escape. v001 won with small gains at the small size and on `hm_0` at the large size. It regressed on `web_0` at the large size, the same weak spot as cycle-0001's gate.

## Patterns

- All three levers act on the same small population: items sitting in tiny, bucket-local LRU stash slots. That makes them interact rather than stack. v001's counter reset on demotion penalizes exactly the demoted items that `Back` placement favors. v003's `Front` insertion behaves like "evict on next collision", so new keys lose their probation hit.
- `web_0` at the large size stayed the hardest cell. No variant recovered it, and every extra filter (gate, comparison, LRU-end insertion) cost hit-rate there. When target buckets have spare room, first-hit promotion is already close to optimal.
- Gains appeared mainly where reuse comes soon or contention is high: small size for the gate, `hm_0` at the large size for v002 and v003. Results split across cells far more often than they moved together.
- Frequency signals are weak in this design. Promotions land at the target's eviction end, and counters are halved or reset on moves, so counters stay near zero. v002's comparison therefore ended up acting like plain first-hit promotion.
- Per-op-time and metadata-size were again not reported. All cost claims are still unverified.

## Open questions

- Does the gate help without regression if it applies only to fresh puts, so demoted items keep their count or promote on their first stash hit?
- Would a frequency comparison actually bite if it compared against the bucket's lowest counter, or if promotions didn't insert at the LRU slot?
- Can fresh-key insertion be softened (one slot above LRU, per-bucket bimodal counter) so it keeps the `hm_0` large-size gain without starving new keys?

## Continuation

Build on this cycle's v001. The top candidate is to restrict the two-hit gate to fresh puts and stop resetting counters on demotion, since that targets the `web_0` large-size regression directly. If frequency admission is retried, fix the victim choice (lowest counter, not LRU index 0) first. Drop hard LRU-end insertion of fresh puts. Make sure per-op-time and metadata-size are measured before any cost claim is made.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle-0002 v001 (MRU-end stash demotion plus the two-hit gate). Each changed one aspect of the gate: v001 changed which items it applies to (demoted items return on one hit), v002 changed when it applies (skipped when the target has a free slot), and v003 changed where items land once they pass it (target MRU end instead of LRU end). The Winner is v003. Its gains came from both 16K cells and `hm_0` at 128K, while `web_0` at 128K stayed flat.

## Patterns

- Where a promotee lands mattered more than tuning the admission rule. With `Front` placement, an item that passed the gate became the target's next eviction victim, so the gate was partly defeating itself. Fixing placement paid off most where target slots are contested.
- v001 disproved the cycle-0002 explanation. Removing the counter-reset penalty on demoted items barely moved `web_0` at 128K. The regression there comes from gating fresh puts when targets have room, not from the reset.
- v002 is the only variant that recovered `web_0` at 128K. v003 gained in the cells v002 lost (16K) and stayed flat in the cell v002 fixed. The two levers cover opposite cells and look complementary.
- Small caches kept penalizing any easier route into the target (v001's one-hit return, v002's free-slot shortcut). In tight caches, a one-hit item that takes the last slot pushes a cascade onto the next promotion. A free slot is not actually free when space is tight.
- The variant insights again report only hit-rate. Per-op-time and metadata-size are still unverified.

## Open questions

- Does v002's occupancy shortcut stack on v003's `Back` placement, recovering `web_0` 128K without giving back the 16K gains?
- Would a stricter room condition (more than one free slot, or an occupancy fraction) remove v002's 16K cost?
- Is v003's flat `web_0` 128K cell caused by promotees jumping to probing on their next hit? Would requiring one more target hit before probing fix it?

## Continuation

Build on v003. The top candidate is to add v002's occupancy-aware gate on top of it, preferably with a stricter room threshold so the 16K cells are protected. A second candidate is delaying a fresh promotee's move to probing. Drop the demoted-item counter lever (v001), since it is near-neutral. Get per-op-time and metadata-size measured before making any cost claims.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle-0003 v003 (`Back` promotion placement plus the two-hit gate). Each tried a different lever: v001 skipped the gate when the target has two free slots, v002 delayed a fresh promotee's move to probing, and v003 changed which stash item gets evicted. The Winner is v001, with a small lift on `web_0` 128K and no regressions. v003 lost hit-rate in every cell.

## Patterns

- The stricter room threshold worked as intended. It kept cycle-0003 v002's `web_0` 128K gain and avoided its 16K loss, which confirms that the last-free-slot cascade caused that loss. With 2-slot buckets, though, the bypass fires only when the target is empty, so this lever has little room left.
- `web_0` 128K is still the bottleneck. This cycle ruled out two explanations for it: one-hit churn into probing (v002 made it worse, which suggests hot promotees want to reach probing sooner) and stash flushing of half-gated items (v003). `hm_0` behaves the opposite way and rewards protecting residents. The two traces keep pulling the timing levers in opposite directions.
- Counters that never decay are now a recurring failure mode. v003's "one hit" protection turned into permanent pinning, and v002's target-hit count only weakly filtered anything. Together with cycle 0002's near-zero counters, this means the `lfu_counter` lifecycle (when it resets, whether it decays) shapes the outcome more than any threshold placed on top of it.
- Plain LRU is strongly confirmed as the right stash eviction rule under the two-hit gate. Fresh demotees especially need the recency protection that `Back` demotion gives them.
- Per-op-time and metadata-size are still unreported, for the fourth cycle in a row.

## Open questions

- Is v002's `hm_0` gain worth keeping through a trace-adaptive or occupancy-conditioned version that leaves `web_0` 128K alone? Or is it too small to pursue?
- Does `web_0` 128K benefit from faster target→probing movement, for example promoting on any target hit and not only at the hottest position?
- Would a stash protection that expires (newest region only, or cleared at the LRU end) gain at 16K, or does any departure from LRU hurt?

## Continuation

Build on this cycle's v001. Drop stash victim-selection by counter and stop tuning the free-slot threshold. For `web_0` 128K, probe the opposite direction from v002: let promotees reach probing more easily. Any new frequency signal must come with an explicit decay or reset rule. Measure per-op-time and metadata-size before making cost claims.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
