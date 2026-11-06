# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's 1%-window winner, and each added one W-TinyLFU component on its own: an SLRU main region (v001), cap-free sketch halving (v002), and a hill-climbing adaptive window (v003). Each one raised the hit-rate summary. The winner is v003's adaptive window. Its gains came mainly at the small size on both traces, which shows the fixed 1% window was too small there.

## Patterns

- The three changes help in different places, so they look more complementary than overlapping. The adaptive window helps most at the small size on both traces. SLRU helps web_0 at both sizes. Aging helps web_0 at both sizes and hm_0 at the large size. On web_0 at the large size, only SLRU and aging improved anything, so the gap left there is not a window-size problem.
- hm_0 benefits little from these changes or is hurt by them. SLRU left it flat. Aging hurt it at the small size, because the reset period counts sketch increments, and a miss produces two increments. Its steady frequent keys get halved before they can build a lead, and halving creates more ties, which the strict `>` rule resolves in favour of the victim.
- Two of the variants depend on a period tied to `capacity`: aging's reset interval and the climber's sample period. Both behave badly at one end of the size range. Aging fires too early at the small size, and the climber moves too slowly at the large size. How these periods are set is now a cross-cutting tuning axis.
- Cycle 0001 suspected the regression came from the 15-cap, not from aging. v002 supports that: aging with no cap no longer regresses on web_0 at the large size.
- Metadata-size and per-op-time are still unmeasured for every variant, even though the previous cycle asked for them. Each variant adds some potential cost: SLRU adds a third list and demotions, aging sweeps the whole table on each reset (latency spikes), and the climber adds per-get sampling.

## Open questions

- Do SLRU and aging add to the adaptive window when combined? Since they help mainly web_0 at the large size, where the climber did nothing, they seem likely to.
- Can a sample period that does not depend on capacity, or is shorter, make the climber useful at the large size?
- Would counting aging periods per request instead of per increment, or using a longer period for small caches, remove the hm_0 small-size regression?
- Should ties in the admission contest be broken differently (for example `>=` or a random tie-break), now that halving makes ties common?

## Continuation

Build on v003, the adaptive window. Next, layer cycle 0002's v001 SLRU split into its main region. Then, as a separate step, add cap-free aging with a per-request period, or a period scaled up for small caches. Try decoupling the climber's sample period from capacity as its own axis. Measure metadata-size and per-op-time before calling any winner, especially the cost of a full-table sketch reset. Drop the fixed 1% window, and drop aging periods counted per sketch increment.
