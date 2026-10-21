# Cycle 0010 Insight

## Cohort outcome

All three variants came from cycle 9's v001 (cap 7 plus `small`-to-`main` carryover):
- **v001** ported the fingerprint-ring ghost, as cycle 9 asked. Hit rate stayed exactly the same and it won. It is now the shared base.
- **v002** aged `main` credits by halving instead of decrementing. Hit rate dipped slightly.
- **v003** reverted the cap to 3 and kept carryover. Hit rate fell by more than its stated tolerance.

## Patterns

- **The combined base is complete.** The fingerprint ring changes how the ghost is stored without changing any ghost decision. It even keeps the parent's stale-entry quirks. So any later hit-rate change can be blamed on policy, not on the ghost's storage format. Its only cost is an extra hash on misses and a negligible chance of fingerprint collisions. Its intended gain, a smaller `metadata-size`, still cannot be scored.
- **Both losses point the same way: `main` gains from keeping credit longer, not from shedding it faster.** Halving cut the buffer that protects keys reused steadily but sparsely. Lowering the cap removed reinsertion passes for keys that stay hot. This fits cycles 7–8: `main`'s slow, forgiving retention is a strength on workload-a. Stale credit is not the bottleneck it was assumed to be.
- **The cap-versus-carryover ablation has a partial answer.** The cap to 7 contributes on its own, so going back to a 2-bit freq costs hit rate. The reverse check (cap 7 with no carryover) has not been run. Both effects remain near the noise floor on one trace at one size.
- **Rejected ideas keep coming back.** v001 and v002 again point to "ghost admissions enter `main` with zero credit" as an open lead. v001 even says cycle 9 recommended it, but cycle 9 dropped it. It lost in cycle 7 and was dropped twice since. Treat it as closed.
- **This is the tenth cycle on one trace, one size and one fitness dimension.** Two of three variants had their real target unscored, or moved hit rate only within noise.

## Open questions

- Does the fingerprint ring actually reduce `metadata-size`? How do S3-FIFO variants compare with the baseline on it and on `per-op-time`?
- Does the gain from the higher cap hold at a second cache size or on a second trace?
- Is carryover worth keeping at all? This needs the mirror ablation: cap 7 with promotion reset to zero.

## Continuation

Use v001, the ring plus cap 7 plus carryover, as the parent for all future work.

Drop these lines of work:
- faster aging in `main`
- lowering the cap or using a 2-bit freq
- ghost-arrival starting credits (closed; do not retry)

The only cheap hit-rate check left is the cap-7, no-carryover ablation, which may simplify the code. Otherwise, do not propose more workload-a/2048 tweaks. The next cycle should get `metadata-size` and `per-op-time` scored, and should run at a second size or on a second trace. If the harness cannot do that, say so plainly.
