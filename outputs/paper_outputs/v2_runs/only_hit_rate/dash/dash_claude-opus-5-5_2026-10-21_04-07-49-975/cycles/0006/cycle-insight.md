# Cycle 0006 Insight

## Cohort outcome

All three variants were built on cycle-0005 v003 (ghost-guided MRU placement), and each probed one of the levers that cycle flagged. v001 made the ghost window 4× larger. v002 gave ghost-admitted keys a one-shot pass through the second-chance scan. v003 made BIP's blind MRU admission 4× rarer. v003 won with a small gain. v002 was barely positive, close to noise, and v001 lost hit-rate.

## Patterns

- The zero-sum rule for protected stash capacity held a third time, now from the other side. The winner did not add protection for anyone. It took blind admissions out of the protected region, and evidence-based residents filled the space. The loser let weaker evidence in: the far-reaching ghost hits that a 4× window produced.
- Evidence counts only if the key can still be reused before MRU placement ages out. A ghost hit about one cache's worth of evictions back predicts reuse. A hit from much further back does not, and those keys displace shorter-reuse residents without paying it back. Long-reuse misses beyond about one cache cannot be recovered by stash placement.
- Once the ghost existed, BIP's random MRU path went from the only way out of probation to a net cost. The two admission paths overlap, and the selective one wins.
- Survival of ghost-admitted keys inside the stash is already solved. MRU placement alone protects nearly all of them, so scan-skip protection adds little. This repeats the 0003 finding that stacked retention mechanisms protecting the same items add nothing.
- Gains keep shrinking, and the levers left inside the stash path look close to tuned out.
- For the fourth cycle running, no writer could inspect the overlay source. Evaluation was again hit-rate only, on one LIRS trace at one size. Per-op-time and metadata-size are still unmeasured, even as ghost and flag metadata build up.

## Open questions

- Does pushing `BIP_PERIOD` further, or removing BIP so the ghost is the only route out of probation, keep improving, or does BIP still catch reuse that the ghost window misses?
- Would a ghost window smaller than one cache, or a ghost admission gated on how recently the fingerprint was evicted, sharpen the evidence signal?
- Do these results hold on traces whose long-reuse keys fall outside the ghost window, and at other cache sizes?

## Continuation

Build on v003 (rarer BIP). Probe the BIP endpoint: a still larger period, or no BIP at all, relying on the ghost alone. Optionally try a smaller ghost window or a recency-gated ghost window. Drop ghost windows larger than about one cache and further scan protection for ghost hits. The stash path is nearly tuned out on this trace. The next cycle should put measuring per-op-time and metadata-size and running a second trace or cache size ahead of more hit-rate micro-tuning.
